use std::{io::Read, ops::Range};

/**
 * A memory-mapped peripheral.
 */
pub trait Device {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, byte: u8);
}

/**
 * Represents the whole range of accessible memory (64k)
 * Registered devices hide the memory below the addresses they occupy
 */
pub struct Memory<'a> {
    mem: [u8; 65536],
    devices: Vec<(Range<usize>, Box<dyn Device + 'a>)>,
}

impl Memory<'_> {
    pub fn new() -> Self {
        Memory {
            mem: [0; 65536],
            devices: Vec::new(),
        }
    }
    pub fn register_device(&mut self, device: Box<dyn Device>, from: usize, to: usize) {
        self.devices.push((from..to+1, device));
    }
    pub fn load_memory_byte(&mut self, addr: u16) -> u8 {
        let addr_ = addr as usize;
        if let Some((Range { start, end: _ }, device)) =
            self.devices.iter_mut().find(|(r, _)| r.contains(&addr_))
        {
            device.read(addr - *start as u16)
        } else {
            self.mem[addr_]
        }
    }
    pub fn load_memory_word(&mut self, addr: u16) -> u16 {
        u16::from_le_bytes([self.load_memory_byte(addr), self.load_memory_byte(addr + 1)])
    }
    pub fn store_memory_byte(&mut self, addr: u16, byte: u8) {
        let addr_ = addr as usize;
        if let Some((Range { start, end: _}, device)) =
         self.devices.iter_mut().find(|(r, _)| r.contains(&addr_)) {
            device.write(addr - *start as u16, byte);
        } else {
            self.mem[addr_] = byte;
        }
    }
    pub fn store_memory_word(&mut self, addr: u16, word: u16) {
        let bytes = word.to_le_bytes();
        self.store_memory_byte(addr, bytes[0]);
        self.store_memory_byte(addr.wrapping_add(1), bytes[1]);
    }
    #[cfg(test)]
    // Note: this does not work for devices
    pub fn get_range(&self, range: Range<usize>) -> &[u8] {
        &self.mem[range]
    }
}

pub struct MemoryFromFile {
    mem: Vec<u8>,
}

impl MemoryFromFile {
    pub fn new(file_name: &str) -> Self {
        let mut f = std::fs::File::open(file_name).unwrap();
        let size: usize = f.metadata().unwrap().len() as usize;
        let mut mem = vec![0_u8; size];
        f.read_exact(&mut mem).unwrap();
        MemoryFromFile { mem }
    }
}

impl Device for MemoryFromFile {
    fn read(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, _addr: u16, _byte: u8) {
        // ignore
    }
}

pub struct MemoryDevice {
    mem: Vec<u8>,
}

impl MemoryDevice {
    pub fn new(size: usize) -> Self {
        MemoryDevice { mem: vec![0; size] }
    }
}

impl Device for MemoryDevice {
    fn read(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, byte: u8) {
        self.mem[addr as usize] = byte;
    }
}
