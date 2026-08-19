use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::cpu6502::device::Device;

pub struct Acia {
    input: VecDeque<u8>,
    command: u8,
    control: u8,
    log_output: Option<Rc<RefCell<dyn FnMut(&str)>>>,
}

impl Acia {
    pub fn new(log_output: Option<Rc<RefCell<dyn FnMut(&str)>>>) -> Self {
        let input = VecDeque::new();
        Self { input, command: 0, control: 0, log_output }
    }
    pub fn set_input(&mut self, s: &str) {
        for c in s.chars() {
            let byte: u8 = c.try_into().unwrap_or(b'?');
            self.input.push_back(byte);
        }
    }
}

impl Device for Acia {
    fn read(&mut self, reg: u8) -> u8 {
        /*
        if self.input.is_empty() {
            let mut s = String::new();
            let _ = std::io::stdin().read_line(&mut s);
            s = s.trim_end().to_string();
            s += "\r\n";
            self.set_input(&s);
        }
        */
        match reg {
            0 => self.input.pop_front().unwrap_or(b'?'),
            // status bit 4: tx data reg empty (always in our case); bit 3: rx data reg full
            1 => if self.input.len() > 0 { 0b00011000 } else { 0b00010000 }
            2 => self.command,
            3 => self.control,
            _ => 0, // should not happen
        }
    }
    fn write(&mut self, reg: u8, byte: u8) {
        match reg {
            0 => {
                if let Some(log) = &self.log_output {
                    let s = format!("{}", char::from(byte));
                    log.borrow_mut()(&s); // data register
                }
            }
            1 => {}, // status register: this soft-resets the chip
            2 => self.command = byte, // command register
            3 => self.control = byte, // control register
            _ => {},
        }
    }
}

