
mod model;
mod execute;

use tracing::info;

use crate::new_c64::cpu::{execute::Execution, model::instruction_and_mode};

use super::memory::Memory;

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub st: StatusFlags,
    pub pc: u16,
    pub sp: u8,
    pub cycle: u8,
    pub cycles: u8,
    pub irq: bool,      // true if the IRQB pin is set to low
    pub nmi: bool,      // true if the NMIB pin is set to low
    pub reset: bool,
    pub tmp: [u8; 2],
    pub tmp_addr: u16,
    pub status_line: String,
    log_instructions: bool,
}

pub struct StatusFlags(pub(crate) u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusFlag {
    Carry,
    Zero,
    IRQDisable,
    Decimal,
    BRK,
    Overflow,
    Negative,
}

impl Into<u8> for StatusFlag {
    fn into(self) -> u8 {
        match self {
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

impl Cpu {
    pub fn new(log_instructions: bool) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            st: StatusFlags(32),
            pc: 0,
            sp: 0xff,
            cycle: 0,
            cycles: 0,
            irq: false,
            nmi: false,
            reset: false,
            tmp: [0, 0],
            tmp_addr: 0,
            status_line: "".to_owned(),
            log_instructions,
        }
    }
    fn trigger() {}
    fn interrupt() {}
    fn nmi() {}
    pub fn reset(&mut self, mem: &Memory) {
        self.pc = mem.read_word(0xfffc);
        self.st = StatusFlags(32);
        self.irq = false;
        self.nmi = false;
        self.cycle = 0;
        self.cycles = 0;
        self.reset = true;
        self.status_line = "(Reset)".to_owned();
    }
    pub fn change_flags(&mut self, enable: &[StatusFlag], disable: &[StatusFlag]) {
        self.set_flags(enable);
        self.clear_flags(disable);
    }
    pub fn set_flag(&mut self, flag: StatusFlag) {
        let flag: u8 = flag.into();
        self.st.0 |= flag;
    }
    pub fn set_flags(&mut self, flags: &[StatusFlag]) {
        for flag in flags {
            self.set_flag(*flag);
        }
    }
    pub fn clear_flag(&mut self, flag: StatusFlag) {
        let flag: u8 = flag.into();
        self.st.0 &= !flag;
    }
    pub fn clear_flags(&mut self, flags: &[StatusFlag]) {
        for flag in flags {
            self.clear_flag(*flag);
        }
    }
    pub fn is_set(&self, flag: StatusFlag) -> bool {
        let flag: u8 = flag.into();
        self.st.0 & flag != 0
    }
    pub fn is_clear(&self, flag: StatusFlag) -> bool {
        let flag: u8 = flag.into();
        self.st.0 & flag == 0
    }
    pub fn set_pc(&mut self) {
        self.pc = self.tmp_addr;
    }
    pub fn inc_pc(&mut self, arg: u8) {
        let arg_signed: i8 = arg.cast_signed();
        self.pc = self.pc.wrapping_add_signed(arg_signed.into());
    }
    pub fn request_interrupt(&mut self) {
        self.irq = true;
    }
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
                let mut execution = Execution::new(self, mem);
                execution.stack_push_pc(2);
                execution.stack_push_flags();
                self.change_flags(&[StatusFlag::IRQDisable], &[StatusFlag::Decimal]);
                self.pc = mem.read_word(0xfffe);
                self.cycles = 7;
                info!("Starting interrupt handler at {:04X}", &self.pc)
            }
            self.irq = false;
            return;
        }
        if self.cycle == 0 {
            let opcode = mem.read(self.pc);
            if self.log_instructions {
                self.status_line = format!(
                    "0b{:08b} a:{:02X} x:{:02X} y:{:02X} 0x{:04X} {}",
                    self.st.0,
                    self.a,
                    self.x,
                    self.y,
                    self.pc,
                    model::disasm(self.pc, mem)
                );
                info!("{}", &self.status_line);
            }
            let (inst, mode) = instruction_and_mode(opcode);
            self.cycles = 0;
            let mut execution = Execution::new(self, mem);
            execution.run_load(inst, mode);
            execution.run_instruction(inst);
            execution.run_store(inst, mode);
            self.cycle += 1;
        } else if self.cycle == self.cycles - 1 {
            self.cycle = 0;
        } else {
            //self.cycle += 1;
            self.cycle = 0;
        }
    }
}
