use std::{io::Read, path::PathBuf};

/**
 * The C64's memory (RAM and ROM)
 * see https://sta.c64.org/cbm64mem.html
 */
pub struct Memory {
    mem: [u8; 65536],
    kernal: [u8; 8192],
    basic: [u8; 8192],
    char_rom: [u8; 4096],
    io: [u8; 4096],
}

impl Memory {
    pub fn new(kernal_file: PathBuf, basic_file: PathBuf) -> Self {
        let mut mem = [0; 65536];
        mem[1] = 0b1110_0000;
        let kernal = {
            let mut f = std::fs::File::open(kernal_file).expect("Unable to open kernal file");
            let mut kernal = [0; 8192];
            f.read_exact(&mut kernal).expect("Unable to read 8192 bytes from kernal file");
            kernal
        };
        let basic = {
            let mut f = std::fs::File::open(basic_file).expect("Unable to open basic file");
            let mut basic = [0; 8192];
            f.read_exact(&mut basic).expect("Unable to read 8192 bytes from basic file");
            basic
        };
        let io = [0; 4096];
        Self { mem, kernal, basic, char_rom: super::char_rom::CHARS, io }
    }
    /**
     * Read a byte from memory at addr.
     * Address 0x0001 determines if we're reading from RAM or one of the ROMs.
     */
    pub fn read(&self, addr: u16) -> u8 {
        if addr < 0xa000 {
            self.mem[addr as usize]
        } else if addr < 0xc000 {
            if self.mem[1] & 0b0110_0000 == 0b0110_0000 {
                self.basic[(addr - 0xa000) as usize]
            } else {
                self.mem[addr as usize]
            }
        } else if addr < 0xe000 {
            if self.mem[1] & 0b0110_0000 == 0 {
                self.mem[addr as usize]
            } else if self.mem[1] & 0b1000_0000 == 0 {
                self.char_rom[(addr - 0xd000) as usize]
            } else {
                self.io[(addr - 0xd000) as usize]
            }
        } else if addr < 0xe000 {
            self.mem[addr as usize]
        } else {
            if self.mem[1] & 0b0100_0000 == 0b0100_0000 {
                self.kernal[(addr - 0xe000) as usize]
            } else {
                self.mem[addr as usize]
            }
        }
    }
    /**
     * Write a byte to memory at addr.
     * If addr is in ROM, ignore.
     */
    pub fn write(&mut self, addr: u16, byte: u8) {
        if addr < 0xa000 {
            self.mem[addr as usize] = byte;
        } else if addr < 0xc000 {
            if self.mem[1] & 0b0110_0000 != 0b0110_0000 {
                self.mem[addr as usize] = byte;
            }
        } else if addr < 0xe000 {
            if self.mem[1] & 0b0110_0000 == 0 {
                self.mem[addr as usize] = byte;
            } else if self.mem[1] & 0b1000_0000 != 0 {
                self.io[(addr - 0xd000) as usize] = byte;
            }
        } else if addr < 0xe000 {
            self.mem[addr as usize] = byte;
        } else {
            if self.mem[1] & 0b0100_0000 != 0b0100_0000 {
                self.mem[addr as usize] = byte;
            }
        }
    }
    pub fn read_word(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read(addr), self.read(addr.wrapping_add(1))])
    }
    pub fn write_word(&mut self, addr: u16, word: u16) {
        let bytes = word.to_le_bytes();
        self.write(addr, bytes[0]);
        self.write(addr.wrapping_add(1), bytes[1]);
    }
}

