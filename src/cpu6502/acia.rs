use std::{cell::RefCell, collections::VecDeque, io::Read, rc::Rc};

use crate::cpu6502::device::Device;

pub struct Acia {
    input: VecDeque<u8>,
    stdin_enabled: bool,
    command: u8,
    control: u8,
    log_output: Option<Rc<RefCell<dyn FnMut(u8)>>>,
}

impl Acia {
    pub fn new(log_output: Option<Rc<RefCell<dyn FnMut(u8)>>>) -> Self {
        let input = VecDeque::new();
        Self {
            input,
            stdin_enabled: true,
            command: 0,
            control: 0,
            log_output,
        }
    }
    pub fn set_input(&mut self, s: &str) {
        for c in s.chars() {
            let byte: u8 = c.try_into().unwrap_or(b'?');
            self.input.push_back(byte);
        }
        self.stdin_enabled = false;
    }
}

impl Device for Acia {
    fn read(&mut self, reg: u8) -> u8 {
        if self.input.is_empty() && self.stdin_enabled {
            let mut buf: [u8; 1] = [0; 1];
            let _ = std::io::stdin().read(&mut buf);
            if buf[0] == b'\n' {
                self.input.push_back(b'\r');
            } else if buf[0] >= b'a' && buf[0] <= b'z' {
                let b = b'A' + (buf[0] - b'a');
                self.input.push_back(b);
            } else {
                self.input.push_back(buf[0]);
            }
        }
        match reg {
            0 => self.input.pop_front().unwrap_or(b'?'),
            // status bit 4: tx data reg empty (always in our case); bit 3: rx data reg full
            1 => {
                if self.input.len() > 0 {
                    0b00011000
                } else {
                    0b00010000
                }
            }
            2 => self.command,
            3 => self.control,
            _ => 0, // should not happen
        }
    }
    fn write(&mut self, reg: u8, byte: u8) {
        match reg {
            0 => {
                if let Some(log) = &self.log_output {
                    log.borrow_mut()(byte); // data register
                }
            }
            1 => {}                   // status register: this soft-resets the chip
            2 => self.command = byte, // command register
            3 => self.control = byte, // control register
            _ => {}
        }
    }
}
