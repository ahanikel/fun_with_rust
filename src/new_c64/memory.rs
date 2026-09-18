use std::{io::Read, path::Path};

use crate::new_c64::{cia1::Cia1, video::Video};

/**
 * The C64's memory (RAM and ROM)
 * see https://sta.c64.org/cbm64mem.html
 */
pub struct Memory {
    mem: [u8; 65536],
    kernal: [u8; 8192],
    basic: [u8; 8192],
    char_rom: [u8; 4096],
}

impl Memory {
    pub fn new(kernal_file: &Path, basic_file: &Path) -> Self {
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
        Self { mem, kernal, basic, char_rom: super::char_rom::CHARS }
    }
    /**
     * Read a byte from memory at addr.
     * Address 0x0001 determines if we're reading from RAM or one of the ROMs.
     */
    pub fn read(&self, video: &Video, cia1: &Cia1, addr: u16) -> u8 {
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
                // IO (Video, SID, CIA1, CIA2)
                if addr < 0xd400 {
                    video.read((addr - 0xd000) % 0x40)
                } else if addr < 0xdc00 {
                    self.mem[addr as usize]
                } else if addr < 0xdd00 {
                    cia1.read((addr - 0xdc00) % 0x10)
                } else if addr < 0xde00 {
                    //self.cia2.write((addr - 0xdc00) % 0x10, byte);
                    self.mem[addr as usize]
                } else {
                    // IO Area #1 and #2
                    self.mem[addr as usize]
                }
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
    pub fn write(&mut self, video: &mut Video, cia1: &mut Cia1, addr: u16, byte: u8) {
        if addr < 0xa000 {
            self.mem[addr as usize] = byte;
        } else if addr < 0xc000 {
            if self.mem[1] & 0b0110_0000 != 0b0110_0000 {
                self.mem[addr as usize] = byte;
            }
        } else if addr < 0xd000 {
            self.mem[addr as usize] = byte;
        } else if addr < 0xe000 {
            if self.mem[1] & 0b0110_0000 == 0 {
                // RAM
                self.mem[addr as usize] = byte;
            } else if self.mem[1] & 0b1000_0000 != 0 {
                // IO (Video, SID, CIA1, CIA2)
                if addr < 0xd400 {
                    video.write((addr - 0xd000) % 0x40, byte);
                } else if addr < 0xdd00 {
                    cia1.write((addr - 0xdc00) % 0x10, byte);
                } else if addr < 0xde00 {
                    //self.cia2.write((addr - 0xdc00) % 0x10, byte);
                }
                // IO Area #1 and #2: ignore
            }
            // Character ROM: ignore
        } else {
            if self.mem[1] & 0b0100_0000 != 0b0100_0000 {
                self.mem[addr as usize] = byte;
            }
        }
    }
    pub fn read_word(&self, video: &Video, cia1: &Cia1, addr: u16) -> u16 {
        u16::from_le_bytes([self.read(video, cia1, addr), self.read(video, cia1, addr.wrapping_add(1))])
    }
    pub fn write_word(&mut self, video: &mut Video, cia1: &mut Cia1, addr: u16, word: u16) {
        let bytes = word.to_le_bytes();
        self.write(video, cia1, addr, bytes[0]);
        self.write(video, cia1, addr.wrapping_add(1), bytes[1]);
    }
}

impl AsRef<Memory> for Memory {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Memory> for Memory {
    fn as_mut(&mut self) -> &mut Memory {
        self
    }
}