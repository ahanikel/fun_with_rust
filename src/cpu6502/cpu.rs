
use crate::cpu6502::memory::Memory;

pub struct CPU<'a> {
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
    pub log_instructions: Option<&'a mut dyn FnMut(&str)>,
}

impl <'a> CPU<'a> {
    pub fn new() -> Self {
        CPU {
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
            log_instructions: None,
        }
    }
    pub fn reset(&mut self, mem: &mut Memory) {
        self.pc = mem.load_memory_word(0xfffc);
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
