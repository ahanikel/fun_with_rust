pub mod acia;
pub mod cpu;
pub mod memory;
pub mod model;
mod test;

use cpu::{CPU, StatusFlag};
use model::instruction_and_mode;
use tracing::info;

use crate::cpu6502::{memory::Memory, model::addr_mode::CpuBusTransfer};

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

    pub fn step(&mut self, mem: &mut Memory) {
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
        if self.cycle == 0 && self.irq {
            if self.is_clear(StatusFlag::IRQDisable) {
                let mut bus_transfer = CpuBusTransfer::new(self, mem);
                bus_transfer.stack_push_pc(2);
                bus_transfer.stack_push_flags();
                self.change_flags(&[StatusFlag::IRQDisable], &[StatusFlag::Decimal]);
                self.pc = mem.load_memory_word(0xfffe);
                self.cycles = 7;
                info!("Starting interrupt handler at {:04X}", &self.pc)
            }
            self.irq = false;
            return;
        }
        if self.cycle == 0 {
            let opcode = mem.load_memory_byte(self.pc);
            if let Some(logger) = &mut self.log_instructions {
                self.status_line = format!(
                    "0b{:08b} a:{:02X} x:{:02X} y:{:02X} 0x{:04X} {}",
                    self.st.0,
                    self.a,
                    self.x,
                    self.y,
                    self.pc,
                    model::disasm(self.pc, mem)
                );
                logger(&self.status_line);
            }
            let (inst, mode) = instruction_and_mode(opcode);
            self.cycles = 0;
            let mut bus_transfer = CpuBusTransfer::new(self, mem);
            bus_transfer.run_load(inst, mode);
            bus_transfer.run_instruction(inst);
            bus_transfer.run_store(inst, mode);
            self.cycle += 1;
        } else if self.cycle == self.cycles - 1 {
            self.cycle = 0;
        } else {
            //self.cycle += 1;
            self.cycle = 0;
        }
    }
}
