use crate::cpu6502::cpu::CPU;

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
        match self {
            AddrMode::AbsoluteIndexedIndirect => false,
            AddrMode::ZeroPageIndirect => false,
            _ => true,
        }
    }
}

impl CPU {
    pub fn stack_push_byte(&mut self) {
        self._stack_push_byte(false);
    }
    pub fn stack_pull_byte(&mut self) {
        self._stack_pull_byte(false);
    }
    pub fn stack_pull_word(&mut self) {
        self._stack_pull_byte(false);
        self._stack_pull_byte(true);
    }
    pub fn stack_push_word(&mut self) {
        self._stack_push_byte(true);
        self._stack_push_byte(false);
    }
    pub fn stack_pull_addr(&mut self) {
        self.stack_pull_word();
        self.tmp_addr = u16::from_le_bytes(self.tmp);
    }
    pub fn stack_push_addr(&mut self) {
        let bytes = self.tmp_addr.to_le_bytes();
        self.tmp[0] = bytes[0];
        self.tmp[1] = bytes[1];
        self.stack_push_word();
    }
    fn _stack_push_byte(&mut self, hi: bool) {
        let byte = self.tmp[if hi {1} else {0}];
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
            self.load_memory_byte_hi(addr);
        } else {
            self.load_memory_byte_lo(addr);
        }
    }
    pub fn stack_push_pc(&mut self, increment: u8) {
        self.tmp_addr = self.pc.wrapping_add(increment.into());
        self.stack_push_addr();
    }
    pub fn stack_push_flags(&mut self) {
        self.tmp[0] = self.st.0;
        self.stack_push_byte();
    }
    pub fn stack_pull_flags(&mut self) {
        self.stack_pull_byte();
        self.st.0 = self.tmp[0];
    }
    fn _load_byte_arg_lo(&mut self) {
        let addr: u16 = self.pc.wrapping_add(1);
        self.load_memory_byte_lo(addr);
    }
    fn _load_byte_arg_hi(&mut self) {
        let addr: u16 = self.pc.wrapping_add(2);
        self.load_memory_byte_hi(addr);
    }
    /**
     * Load the byte-sized argument into tmp[0]
     * tmp[1] is set to 0
     */
    pub fn load_byte_arg(&mut self) {
        self.tmp[1] = 0;
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
        self.tmp_addr = u16::from_le_bytes(self.tmp);
    }
    /**
     * Load the zeropage address argument into tmp_addr
     */
    pub fn load_zp_arg(&mut self) {
        self.load_byte_arg();
        self.tmp_addr = u16::from_le_bytes(self.tmp);
    }
    fn _load_absolute_lo(&mut self) {
        self.load_addr_arg();
        self.load_memory_byte_lo(self.tmp_addr);
    }
    fn _load_absolute_hi(&mut self) {
        self.load_memory_byte_hi(self.tmp_addr);
    }
    /**
     * Loads a byte from the address pointed at by the address argument
     */
    pub fn load_absolute_byte(&mut self) {
        self._load_absolute_lo()
    }
    /**
     * Loads a word from the address pointed at by the argument
     */
    pub fn load_absolute_word(&mut self) {
        self._load_absolute_lo();
        self._load_absolute_hi();
    }
    /**
     * Loads an address from the address pointed at by the argument
     */
    pub fn load_absolute_addr(&mut self) {
        self.load_absolute_word();
        self.tmp_addr = u16::from_le_bytes(self.tmp);
    }
    /**
     * Loads a byte from (abs)
     */
    pub fn load_absolute_indirect_byte(&mut self) {
        self.load_absolute_addr();
        self.load_memory_byte_lo(self.tmp_addr);
    }
    /**
     * Loads a byte from (abs,x)
     */
    pub fn load_absolute_indexed_indirect_byte(&mut self) {
        self.load_addr_arg();
        self.tmp_addr = self.tmp_addr.wrapping_add(self.x.into());
        self.load_memory_addr(self.tmp_addr);
        self.load_memory_byte_lo(self.tmp_addr);
    }
    /**
     * Loads a byte from abs,x
     */
    pub fn load_absolute_indexed_with_x_byte(&mut self) {
        self.load_addr_arg();
        self.tmp_addr = self.tmp_addr.wrapping_add(self.x.into());
        self.load_memory_byte_lo(self.tmp_addr);
    }
     /**
     * Loads a byte from a,y
     */
    pub fn load_absolute_indexed_with_y_byte(&mut self) {
        self.load_addr_arg();
        self.tmp_addr = self.tmp_addr.wrapping_add(self.y.into());
        self.load_memory_byte_lo(self.tmp_addr);
    }
    /**
     * Loads a byte from the (zp,x) address in the argument
     */
    pub fn load_zp_indexed_indirect_byte(&mut self) {
        self.load_zp_arg();
        self.tmp_addr = self.tmp_addr.wrapping_add(self.x.into());
        self.load_memory_addr(self.tmp_addr);
        self.load_memory_byte_lo(self.tmp_addr);
    }
    /**
     * Load a byte from the (zp) address in the argument
     */
    pub fn load_zp_indirect_byte(&mut self) {
        self.load_zp_arg();
        self.load_memory_addr(self.tmp_addr);
        self.load_memory_byte_lo(self.tmp_addr);
    }
    /**
     * Load a byte from the (zp),y address in the argument
     */
    pub fn load_zp_indirect_indexed_with_y_byte(&mut self) {
        self.load_zp_indirect_byte();
        self.tmp[0] = self.tmp[0] + self.y;
    }
    /**
     * Load a byte from the zp address in the argument
     */
    pub fn load_zp_byte(&mut self) {
        self.load_zp_arg();
        self.load_memory_byte_lo(self.tmp_addr);
    }
    /**
     * Load a byte from the zp,x address in the argument
     */
     pub fn load_zp_indexed_with_x_byte(&mut self) {
        self.load_zp_arg();
        self.tmp_addr = self.tmp_addr.wrapping_add(self.x.into());
        self.load_memory_byte_lo(self.tmp_addr);
    }
    /**
     * Load a byte from the zp,y address in the argument
     */
     pub fn load_zp_indexed_with_y_byte(&mut self) {
        self.load_zp_arg();
        self.tmp_addr = self.tmp_addr.wrapping_add(self.y.into());
        self.load_memory_byte_lo(self.tmp_addr);
    }
}
 