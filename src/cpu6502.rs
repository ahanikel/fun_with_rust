pub mod cpu;
pub mod model;
mod test;

use cpu::{CPU, Cycle, StatusFlag};
use model::{addr_mode::AddrMode, instruction::Instruction, instruction_and_mode};


impl CPU {
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

            (Instruction::BRK, AddrMode::Stack, 2) if self.is_set(StatusFlag::BRK) => Cycle(3),
            (Instruction::BRK, AddrMode::Stack, 3) if self.is_set(StatusFlag::BRK) => {
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
                self.load_byte_arg();
                self.a = self.a | self.tmp[0];
                self.check_and_set_nz_flags(self.a);
                self.inc_pc(2)
            }

            (Instruction::ASL, AddrMode::Accumulator, _) => Cycle(0),

            (Instruction::TSB, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::TSB, AddrMode::Absolute, 5) => {
                self.check_and_set_z_flag(self.a & self.tmp[0]);
                self.store_memory_byte(self.tmp_addr, self.a | self.tmp[0]);
                self.inc_pc(3)
            }

            (Instruction::ORA, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::ORA, AddrMode::Absolute, 5) => {
                self.a = self.a | self.tmp[0];
                self.check_and_set_nz_flags(self.a);
                self.inc_pc(3)
            }

            (Instruction::ASL, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::ASL, AddrMode::Absolute, 5) => {
                if self.tmp[0] >= 0x80 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }
                let ret = self.tmp[0] << 1;
                self.check_and_set_nz_flags(ret);
                self.store_memory_byte(self.tmp_addr, ret);
                self.inc_pc(3)
            }

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

            (Instruction::TRB, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::TRB, AddrMode::Absolute, 5) => {
                self.check_and_set_z_flag(self.a & self.tmp[0]);
                self.store_memory_byte(self.tmp_addr, !self.a & self.tmp[0])
            }

            (Instruction::ORA, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::ASL, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::BBR1, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::JSR, AddrMode::Absolute, 2) => self.stack_push_pc(3),
            (Instruction::JSR, AddrMode::Absolute, 4) => self.load_addr_arg(),
            (Instruction::JSR, AddrMode::Absolute, 6) => self.set_pc(),

            (Instruction::AND, AddrMode::ZeroPageIndexedIndirect, _) => Cycle(0),
            (Instruction::BIT, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::AND, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::ROL, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::RMB2, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::PLP, AddrMode::Stack, _) => Cycle(0),

            (Instruction::AND, AddrMode::Immediate, 2) => {
                self.load_byte_arg();
                self.a = self.a & self.tmp[0];
                self.check_and_set_nz_flags(self.a);
                self.inc_pc(2)
            }

            (Instruction::ROL, AddrMode::Accumulator, _) => Cycle(0),

            (Instruction::BIT, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::BIT, AddrMode::Absolute, 5) => {
                let test = self.a & self.tmp[0];
                self.check_and_set_nz_flags(test);
                if test & 0x40 == 0 {
                    self.clear_flag(StatusFlag::Overflow);
                } else {
                    self.set_flag(StatusFlag::Overflow);
                }
                self.inc_pc(3)
            }

            (Instruction::AND, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::AND, AddrMode::Absolute, 5) => {
                self.a = self.a & self.tmp[0];
                self.check_and_set_nz_flags(self.a);
                self.inc_pc(3)
            }

            (Instruction::ROL, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::ROL, AddrMode::Absolute, 5) => {
                let old_carry = self.is_set(StatusFlag::Carry);
                if self.tmp[0] & 0x80 == 0x80 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }
                let res = self.tmp[0] << 1;
                let res = res | if old_carry {1} else {0};
                self.store_memory_byte(self.tmp_addr, res);
                self.inc_pc(3)
            }

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

            (Instruction::DEC, AddrMode::Accumulator, 2) => {
                if self.a == 0 {
                    self.a = 0xff;
                } else {
                    self.a = self.a - 1;
                }
                self.check_and_set_nz_flags(self.a);
                self.inc_pc(2)
            }

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

            (Instruction::JMP, AddrMode::Absolute, 2) => self.load_addr_arg(),
            (Instruction::JMP, AddrMode::Absolute, 4) => self.set_pc(),

            (Instruction::EOR, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::EOR, AddrMode::Absolute, 5) => {
                self.a = self.a ^ self.tmp[0];
                self.check_and_set_nz_flags(self.a);
                self.inc_pc(3)
            }
 
            (Instruction::LSR, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::LSR, AddrMode::Absolute, 5) => {
                if self.tmp[0] & 0x1 == 1 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }
                let res = self.tmp[0] >> 1;
                self.store_memory_byte(self.tmp_addr, res);
                self.inc_pc(3)
            }

            (Instruction::BBR4, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::BVC, AddrMode::ProgramCounterRelative, 2)
                if self.is_clear(StatusFlag::Overflow) =>
            {
                self.load_memory_byte_lo(Self::addr_add(self.pc, 1))
            }
            (Instruction::BVC, AddrMode::ProgramCounterRelative, 3)
                if self.is_clear(StatusFlag::Overflow) =>
            {
                self.inc_pc(self.tmp[0])
            }
            (Instruction::BVC, AddrMode::ProgramCounterRelative, 2) => self.inc_pc(2),

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

            (Instruction::PHY, AddrMode::Stack, 2) => self.stack_push_byte(self.y),
            (Instruction::PHY, AddrMode::Stack, 3) => self.inc_pc(1),

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
                self.check_and_set_nz_flags(self.a);
                self.inc_pc(2)
            }

            (Instruction::ADC, AddrMode::Immediate, _) => Cycle(0),
            (Instruction::ROR, AddrMode::Accumulator, _) => Cycle(0),
            (Instruction::JMP, AddrMode::AbsoluteIndirect, _) => Cycle(0),

            (Instruction::ADC, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::ADC, AddrMode::Absolute, 5) => {
                // add the two numbers
                let a = self.a;
                let b = self.tmp[0];
                let res = a.wrapping_add(b);
                // An overflow occurs if and only if two numbers with the same sign are added,
                // but the result has the opposite sign:
                let ofl1 = (a ^ res) & (b ^ res) & 0x80 != 0;

                // add the carry
                let a = res;
                let b: u8 = if self.is_set(StatusFlag::Carry) {1} else {0};
                let res = a.wrapping_add(b);
                let ofl2 = (a ^ res) & (b ^ res) & 0x80 != 0;
                self.set_or_clear_flag(StatusFlag::Overflow, ofl1 || ofl2);

                self.check_and_set_nz_flags(res);
                self.a = res;
                self.inc_pc(3)
            }

            (Instruction::ROR, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::ROR, AddrMode::Absolute, 5) => {
                let old_carry = self.is_set(StatusFlag::Carry);
                let b0 = self.tmp[0] | 1;
                self.check_and_set_or_clear_flag(StatusFlag::Carry, b0);
                let mut res = self.tmp[0] >> 1;
                if old_carry {
                    res = res | 0x80;
                }
                self.check_and_set_nz_flags(res);
                self.store_memory_byte(self.tmp_addr, res);
                self.inc_pc(3)
            }

            (Instruction::BBR6, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::BVS, AddrMode::ProgramCounterRelative, 2)
                if self.is_set(StatusFlag::Overflow) =>
            {
                self.load_memory_byte_lo(Self::addr_add(self.pc, 1))
            }
            (Instruction::BVS, AddrMode::ProgramCounterRelative, 3)
                if self.is_set(StatusFlag::Overflow) =>
            {
                self.inc_pc(self.tmp[0])
            }
            (Instruction::BVS, AddrMode::ProgramCounterRelative, 2) => self.inc_pc(2),

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
                self.check_and_set_nz_flags(self.y);
                self.inc_pc(2)
            }

            (Instruction::JMP, AddrMode::AbsoluteIndexedIndirect, _) => Cycle(0),
            (Instruction::ADC, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::ROR, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::BBR7, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::BRA, AddrMode::ProgramCounterRelative, 2) => {
                self.load_memory_byte_lo(Self::addr_add(self.pc, 1))
            }
            (Instruction::BRA, AddrMode::ProgramCounterRelative, 3) => self.inc_pc(self.tmp[0]),

            (Instruction::STA, AddrMode::ZeroPageIndexedIndirect, _) => Cycle(0),
            (Instruction::STY, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::STA, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::STX, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::SMB0, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::DEY, AddrMode::Implied, 2) => {
                if self.y == 0 {
                    self.y = 0xff;
                } else {
                    self.y = self.y - 1;
                }
                self.check_and_set_nz_flags(self.y);
                self.inc_pc(1)
            }

            (Instruction::BIT, AddrMode::Immediate, _) => Cycle(0),
            (Instruction::TXA, AddrMode::Implied, 2) => {
                self.a = self.x;
                self.check_and_set_nz_flags(self.a);
                self.inc_pc(1)
            }

            (Instruction::STY, AddrMode::Absolute, 2) => self.load_addr_arg(),
            (Instruction::STY, AddrMode::Absolute, 4) => {
                self.store_memory_byte(self.tmp_addr, self.y);
                self.inc_pc(3)
            }

            (Instruction::STA, AddrMode::Absolute, 2) => self.load_addr_arg(),
            (Instruction::STA, AddrMode::Absolute, 4) => {
                self.store_memory_byte(self.tmp_addr, self.a);
                self.inc_pc(3)
            }

            (Instruction::STX, AddrMode::Absolute, 2) => self.load_addr_arg(),
            (Instruction::STX, AddrMode::Absolute, 4) => {
                self.store_memory_byte(self.tmp_addr, self.x);
                self.inc_pc(3)
            }

            (Instruction::BBS0, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::BCC, AddrMode::ProgramCounterRelative, 2)
                if self.is_clear(StatusFlag::Carry) =>
            {
                self.load_memory_byte_lo(Self::addr_add(self.pc, 1))
            }
            (Instruction::BCC, AddrMode::ProgramCounterRelative, 3)
                if self.is_clear(StatusFlag::Carry) =>
            {
                self.inc_pc(self.tmp[0])
            }
            (Instruction::BCC, AddrMode::ProgramCounterRelative, 2) => self.inc_pc(2),

            (Instruction::STA, AddrMode::ZeroPageIndirectIndexedWithY, _) => Cycle(0),
            (Instruction::STA, AddrMode::ZeroPageIndirect, _) => Cycle(0),
            (Instruction::STY, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::STA, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::STX, AddrMode::ZeroPageIndexedWithY, _) => Cycle(0),
            (Instruction::SMB1, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::TYA, AddrMode::Implied, 2) => {
                self.a = self.y;
                self.check_and_set_nz_flags(self.a);
                self.inc_pc(1)
            }

            (Instruction::STA, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),

            (Instruction::TXS, AddrMode::Implied, 2) => {
                self.sp = self.x;
                self.inc_pc(1)
            }

            (Instruction::STZ, AddrMode::Absolute, 2) => self.load_addr_arg(),
            (Instruction::STZ, AddrMode::Absolute, 4) => {
                self.store_memory_byte(self.tmp_addr, 0);
                self.inc_pc(3)
            }

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

            (Instruction::TAY, AddrMode::Implied, 2) => {
                self.y = self.a;
                self.check_and_set_nz_flags(self.y);
                self.inc_pc(1)
            }

            (Instruction::LDA, AddrMode::Immediate, _) => Cycle(0),

            (Instruction::TAX, AddrMode::Implied, 2) => {
                self.x = self.a;
                self.check_and_set_nz_flags(self.x);
                self.inc_pc(1)
            }

            (Instruction::LDY, AddrMode::Accumulator, _) => Cycle(0),

            (Instruction::LDA, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::LDA, AddrMode::Absolute, 5) => {
                self.a = self.tmp[0];
                self.check_and_set_nz_flags(self.a);
                self.inc_pc(3)
            }

            (Instruction::LDX, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::LDX, AddrMode::Absolute, 5) => {
                self.a = self.tmp[0];
                self.check_and_set_nz_flags(self.a);
                self.inc_pc(3)
            }

            (Instruction::BBS2, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::BCS, AddrMode::ProgramCounterRelative, 2)
                if self.is_set(StatusFlag::Carry) =>
            {
                self.load_memory_byte_lo(Self::addr_add(self.pc, 1))
            }
            (Instruction::BCS, AddrMode::ProgramCounterRelative, 3)
                if self.is_set(StatusFlag::Carry) =>
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

            (Instruction::CLV, AddrMode::Implied, 2) => {
                self.clear_flag(StatusFlag::Overflow);
                self.inc_pc(1)
            }

            (Instruction::LDA, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),

            (Instruction::TSX, AddrMode::Implied, 2) => {
                self.x = self.sp;
                self.check_and_set_nz_flags(self.x);
                self.inc_pc(1)
            }

            (Instruction::LDY, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::LDA, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::LDX, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),
            (Instruction::BBS3, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::CPY, AddrMode::Immediate, _) => Cycle(0),

            (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect, 2) => self.load_byte_arg(),
            (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect, 3) => self.load_indexed_x_lo(),
            (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect, 4) => self.load_indexed_x_hi(),
            (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect, 5) => {
                self.load_memory_byte_lo(self.tmp_addr)
            }
            (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect, 6) => {
                self.compare_and_set_flags(self.a, self.tmp[0]);
                self.inc_pc(2)
            }

            (Instruction::CPY, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::CMP, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::DEC, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::SMB4, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::INY, AddrMode::Implied, 2) => {
                if self.y == 0xff {
                    self.y = 0;
                } else {
                    self.y = self.y + 1;
                }
                self.check_and_set_nz_flags(self.y);
                self.inc_pc(1)
            }

            (Instruction::CMP, AddrMode::Immediate, 2) => self.load_byte_arg(),
            (Instruction::CMP, AddrMode::Immediate, 3) => {
                self.compare_and_set_flags(self.a, self.tmp[0]);
                self.inc_pc(2)
            }

            (Instruction::DEX, AddrMode::Implied, 2) => {
                if self.x == 0 {
                    self.x = 0xff;
                } else {
                    self.x = self.x - 1;
                }
                self.check_and_set_nz_flags(self.x);
                self.inc_pc(2)
            }

            (Instruction::WAI, AddrMode::Implied, 2) => Cycle(0), // busy waiting for now

            (Instruction::CPY, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::CPY, AddrMode::Absolute, 5) => {
                self.compare_and_set_flags(self.y, self.tmp[0]);
                self.inc_pc(3)
            }

            (Instruction::CMP, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::CMP, AddrMode::Absolute, 5) => {
                self.compare_and_set_flags(self.a, self.tmp[0]);
                self.inc_pc(3)
            }

            (Instruction::DEC, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::DEC, AddrMode::Absolute, 5) => {
                self.tmp[0] = self.tmp[0].wrapping_sub(1);
                self.store_memory_byte(self.tmp_addr, self.tmp[0]);
                self.check_and_set_nz_flags(self.tmp[0]);
                self.inc_pc(3)
            }

            (Instruction::BBS4, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::BNE, AddrMode::ProgramCounterRelative, 2)
                if self.is_clear(StatusFlag::Zero) =>
            {
                self.load_memory_byte_lo(Self::addr_add(self.pc, 1))
            }
            (Instruction::BNE, AddrMode::ProgramCounterRelative, 3)
                if self.is_clear(StatusFlag::Zero) =>
            {
                self.inc_pc(self.tmp[0])
            }
            (Instruction::BNE, AddrMode::ProgramCounterRelative, 2) => self.inc_pc(2),

            (Instruction::CMP, AddrMode::ZeroPageIndirectIndexedWithY, _) => Cycle(0),
            (Instruction::CMP, AddrMode::ZeroPageIndirect, _) => Cycle(0),
            (Instruction::CMP, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::DEC, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::SMB5, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::CLD, AddrMode::Implied, 2) => {
                self.clear_flag(StatusFlag::Decimal);
                self.inc_pc(1)
            }

            (Instruction::CMP, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),

            (Instruction::PHX, AddrMode::Stack, 2) => self.stack_push_byte(self.x),
            (Instruction::PHX, AddrMode::Stack, 3) => self.inc_pc(1),

            (Instruction::STP, AddrMode::Implied, 2) => Cycle(0), // busy waiting for now

            (Instruction::CMP, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::DEC, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::BBS5, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::CPX, AddrMode::Immediate, _) => Cycle(0),
            (Instruction::SBC, AddrMode::ZeroPageIndexedIndirect, _) => Cycle(0),
            (Instruction::CPX, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::SBC, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::INC, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::SMB6, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::INX, AddrMode::Implied, 2) => {
                if self.x == 0xff {
                    self.x = 0;
                } else {
                    self.x = self.x + 1;
                }
                self.check_and_set_nz_flags(self.x);
                self.inc_pc(1)
            }

            (Instruction::SBC, AddrMode::Immediate, _) => Cycle(0),

            (Instruction::NOP, AddrMode::Implied, 2) => self.inc_pc(1),

            (Instruction::CPX, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::CPX, AddrMode::Absolute, 5) => {
                self.compare_and_set_flags(self.x, self.tmp[0]);
                self.inc_pc(3)
            }

            (Instruction::SBC, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::SBC, AddrMode::Absolute, 5) => {
                // add the accumulator and the complement of the argument
                let a = self.a;
                let b = !self.tmp[0];
                let res = a.wrapping_add(b);
                // An overflow occurs if and only if two numbers with the same sign are added,
                // but the result has the opposite sign:
                let ofl1 = (a ^ res) & (b ^ res) & 0x80 != 0;

                // add the carry
                let a = res;
                let b: u8 = if self.is_set(StatusFlag::Carry) {1} else {0};
                let res = a.wrapping_add(b);
                let ofl2 = (a ^ res) & (b ^ res) & 0x80 != 0;
                self.set_or_clear_flag(StatusFlag::Overflow, ofl1 || ofl2);

                self.check_and_set_nz_flags(res);
                self.a = res;
                self.inc_pc(3)
            }

            (Instruction::INC, AddrMode::Absolute, 2) => self.load_absolute_byte(),
            (Instruction::INC, AddrMode::Absolute, 5) => {
                self.tmp[0] = self.tmp[0].wrapping_add(1);
                self.store_memory_byte(self.tmp_addr, self.tmp[0]);
                self.check_and_set_nz_flags(self.tmp[0]);
                self.inc_pc(3)
            }


            (Instruction::BBS6, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::BEQ, AddrMode::ProgramCounterRelative, 2)
                if self.is_set(StatusFlag::Zero) =>
            {
                self.load_memory_byte_lo(Self::addr_add(self.pc, 1))
            }
            (Instruction::BEQ, AddrMode::ProgramCounterRelative, 3)
                if self.is_set(StatusFlag::Zero) =>
            {
                self.inc_pc(self.tmp[0])
            }
            (Instruction::BEQ, AddrMode::ProgramCounterRelative, 2) => self.inc_pc(2),

            (Instruction::SBC, AddrMode::ZeroPageIndirectIndexedWithY, _) => Cycle(0),
            (Instruction::SBC, AddrMode::ZeroPageIndirect, _) => Cycle(0),
            (Instruction::SBC, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::INC, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::SMB7, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::SED, AddrMode::Implied, 2) => {
                self.set_flag(StatusFlag::Decimal);
                self.inc_pc(1)
            }

            (Instruction::SBC, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),

            (Instruction::PLX, AddrMode::Stack, 2) => self.stack_pull_byte_lo(),
            (Instruction::PLX, AddrMode::Stack, 3) => {
                self.x = self.tmp[0];
                self.check_and_set_nz_flags(self.x);
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
