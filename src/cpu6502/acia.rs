use crate::cpu6502::device::Device;

pub struct Acia {
    input: Vec<u8>,
    command: u8,
    control: u8,
}

impl Acia {
    pub fn new(input: String) -> Self {
        Acia { input: input.into_bytes(), command: 0, control: 0 }
    }
}

impl Device for Acia {
    fn read(&mut self, reg: u8) -> u8 {
        match reg {
            0 => self.input.pop().unwrap_or(0),
            // status bit 4: tx data reg empty (always in our case); bit 3: rx data reg full
            1 => if self.input.len() > 0 { 0b00011000 } else { 0b00010000 }
            2 => self.command,
            3 => self.control,
            _ => 0, // should not happen
        }
    }
    fn write(&mut self, reg: u8, byte: u8) {
        match reg {
            0 => print!("{}", char::from(byte)), // data register
            1 => {}, // status register: this soft-resets the chip
            2 => self.command = byte, // command register
            3 => self.control = byte, // control register
            _ => {},
        }
    }
}
