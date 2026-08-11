pub struct CPU {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub st: StatusFlags,
    pub pc: u16,
    pub sp: u8,
    pub cycle: u8,
    pub cycles: u8,
    pub mem: [u8; 65536],
    pub irq: bool,      // true if the IRQB pin is set to low
    #[allow(unused)]
    pub irq_prev: bool, // previous state of the IRQB pin to detect negative transition
    pub nmi: bool,      // true if the NMIB pin is set to low
    #[allow(unused)]
    pub nmi_prev: bool, // previous state of the NMIB pin to detect negative transition
    pub reset: bool,
    pub tmp: [u8; 2],
    pub tmp_addr: u16,
    pub status_line: String,
}

impl CPU {
    pub fn new() -> Self {
        let mem: [u8; 65536] = [0; 65536];
        CPU {
            a: 0,
            x: 0,
            y: 0,
            st: StatusFlags(32),
            pc: 0,
            sp: 0xff,
            cycle: 0,
            cycles: 0,
            mem,
            irq: false,
            irq_prev: false,
            nmi: false,
            nmi_prev: false,
            reset: false,
            tmp: [0, 0],
            tmp_addr: 0,
            status_line: "".to_owned(),
        }
    }
    pub fn reset(&mut self) {
        self.load_memory_byte_lo(0xfffc);
        self.load_memory_byte_hi(0xfffd);
        self.pc = u16::from_le_bytes(self.tmp);
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
        self.st.0 = self.st.0 | flag;
    }
    pub fn set_flags(&mut self, flags: &[StatusFlag]) {
        for flag in flags {
            self.set_flag(*flag);
        }
    }
    pub fn clear_flag(&mut self, flag: StatusFlag) {
        let flag: u8 = flag.into();
        self.st.0 = self.st.0 & !flag;
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
    pub fn load_memory_byte_lo(&mut self, addr: u16) {
        // TODO: insert code for peripherals
        let addr: usize = addr.into();
        self.tmp[0] = self.mem[addr];
    }
    pub fn load_memory_byte_hi(&mut self, addr: u16) {
        // TODO: insert code for peripherals
        let addr: usize = addr.into();
        self.tmp[1] = self.mem[addr];
    }
    /**
     * Load a word at address addr into self.tmp
     */
    pub fn load_memory_word(&mut self, addr: u16) {
        self.load_memory_byte_lo(addr);
        self.load_memory_byte_hi(addr + 1);
    }
    pub fn store_memory_byte(&mut self, addr: u16, byte: u8) {
        let addr: usize = addr.into();
        self.mem[addr] = byte;
    }
    /**
     * Load an address at address addr into self.tmp_addr
     */
    pub fn load_memory_addr(&mut self, addr: u16) {
        self.load_memory_word(addr);
        self.tmp_addr = u16::from_le_bytes(self.tmp);
    }
 }

pub struct StatusFlags(pub(crate) u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusFlag {
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
