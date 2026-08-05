use crate::cpu6502::cpu::{CPU, Cycle};

#[derive(Debug, Clone, Copy)]
pub enum AddrMode {
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

impl super::IsOriginal for AddrMode {
    fn is_original(&self) -> bool {
        match self {
            AddrMode::AbsoluteIndexedIndirect => false,
            AddrMode::ZeroPageIndirect => false,
            _ => true,
        }
    }
}

impl CPU {
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
    pub fn stack_push_byte(&mut self, byte: u8) -> Cycle {
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
    pub fn stack_pull_byte_lo(&mut self) -> Cycle {
        self._stack_pull_byte(false);
        self.cycle.plus(1)
    }
    pub fn stack_pull_byte_hi(&mut self) -> Cycle {
        self._stack_pull_byte(true);
        self.cycle.plus(1)
    }
    pub fn addr_add(addr: u16, val: u8) -> u16 {
        let ret: u32 = addr.into();
        let val: u32 = val.into();
        let mut ret = ret + val;
        if ret > 0xffff {
            ret = ret - 0x10000;
        }
        let ret: u16 = u16::try_from(ret).unwrap();
        ret
    }
    pub fn stack_push_pc_lo(&mut self, increment: u8) -> Cycle {
        let pc = Self::addr_add(self.pc, increment);
        let lo: u8 = (pc & 0xff).try_into().unwrap();
        self.stack_push_byte(lo)
    }
    pub fn stack_push_pc_hi(&mut self, increment: u8) -> Cycle {
        let pc = Self::addr_add(self.pc, increment);
        let hi: u8 = (pc >> 8).try_into().unwrap();
        self.stack_push_byte(hi)
    }
    pub fn stack_push_pc(&mut self, increment: u8) -> Cycle {
        self.stack_push_pc_hi(increment)
        .fplus(self.stack_push_pc_lo(increment))
    }
    pub fn stack_push_flags(&mut self) -> Cycle {
        self.stack_push_byte(self.st.0)
    }
    pub fn stack_pull_flags(&mut self) -> Cycle {
        self.stack_pull_byte_lo();
        self.st.0 = self.tmp[0];
        self.cycle.plus(1)
    }
    fn _load_byte_arg_lo(&mut self) -> Cycle {
        let addr: u16 = Self::addr_add(self.pc, 1);
        self.load_memory_byte_lo(addr)
    }
    fn _load_byte_arg_hi(&mut self) -> Cycle {
        let addr: u16 = Self::addr_add(self.pc, 2);
        self.load_memory_byte_hi(addr)
    }
    /**
     * Load the byte-sized argument into tmp[0]
     * tmp[1] is set to 0
     * Takes 1 cycle
     */
    pub fn load_byte_arg(&mut self) -> Cycle {
        self.tmp[1] = 0;
        self._load_byte_arg_lo()
    }
    /**
     * Load the word-sized argument into tmp
     * Takes 2 cycles
     */
    pub fn load_word_arg(&mut self) -> Cycle {
        self._load_byte_arg_lo()
        .fplus(self._load_byte_arg_hi())
    }
    /**
     * Load the address argument into tmp_addr
     * Takes 2 cycles
     */
    pub fn load_addr_arg(&mut self) -> Cycle {
        let ret = self.load_word_arg();
        self.tmp_addr = u16::from_le_bytes(self.tmp);
        ret
    }
    pub fn load_indexed_x_lo(&mut self) -> Cycle {
        let addr: u16 = u16::from_le_bytes(self.tmp);
        self.tmp_addr = Self::addr_add(addr, self.x) & 0xff;
        self.load_memory_byte_lo(self.tmp_addr)
    }
    pub fn load_indexed_x_hi(&mut self) -> Cycle {
        let ret = self.load_memory_byte_hi(self.tmp_addr);
        self.tmp_addr = u16::from_le_bytes(self.tmp);
        ret
    }
    fn _load_absolute_lo(&mut self) -> Cycle {
        self.load_addr_arg()
            .fplus({
                self.load_memory_byte_lo(self.tmp_addr)
            })
    }
    fn _load_absolute_hi(&mut self) -> Cycle {
        self.load_memory_byte_hi(self.tmp_addr)
    }
    /**
     * Loads a byte from the address pointed at by the address argument
     * Takes 3 cycles
     */
    pub fn load_absolute_byte(&mut self) -> Cycle {
        self._load_absolute_lo()
    }
    /**
     * Loads a word from the address pointed at by the address argument
     * Takes 4 cycles
     */
    #[allow(dead_code)]
    pub fn load_absolute_word(&mut self) -> Cycle {
        self._load_absolute_lo()
        .fplus(self._load_absolute_hi())
    }
}
 