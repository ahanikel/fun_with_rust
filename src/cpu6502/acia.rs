use std::{
    cell::RefCell, collections::VecDeque, rc::Rc, sync::{Arc, RwLock}, thread::{self, JoinHandle},
};

use crate::cpu6502::memory::Device;

#[allow(clippy::type_complexity)]
pub struct Acia {
    input: Arc<RwLock<VecDeque<u8>>>,
    stdin_enabled: bool,
    command: u8,
    control: u8,
    log_output: Option<Rc<RefCell<dyn FnMut(u8)>>>,
    pub input_thread: Option<JoinHandle<()>>,
}

#[allow(clippy::type_complexity)]
impl Acia {
    pub fn new(log_output: Option<Rc<RefCell<dyn FnMut(u8)>>>) -> Self {
        let input = Arc::new(RwLock::new(VecDeque::new()));
        Self {
            input,
            stdin_enabled: true,
            command: 0,
            control: 0,
            log_output,
            input_thread: None,
        }
    }
    pub fn start(&mut self) {
        let input = self.input.clone();
        if self.stdin_enabled {
            self.input_thread = Some(thread::spawn(move || {
                info!("ACIA background thread starting.");
                loop {
                    if let Some(c) = read_char_non_blocking() {
                        if c == b'\n' {
                            input.write().unwrap().push_back(b'\r');
                        } else if c == b'q' {
                            break;
                        } else if c >= b'a' && c <= b'z' {
                            let b = b'A' + (c - b'a');
                            input.write().unwrap().push_back(b);
                        } else {
                            input.write().unwrap().push_back(c);
                        }
                    }
                }
                info!("ACIA background thread terminating.");
            }))
        };
    }
    #[cfg(test)]
    pub fn set_input(&mut self, s: &str) {
        for c in s.chars() {
            let byte: u8 = c.try_into().unwrap_or(b'?');
            if byte >= b'a' && byte <= b'z' {
                let b = b'A' + (byte - b'a');
                self.input.write().unwrap().push_back(b);
            } else {
                self.input.write().unwrap().push_back(byte);
            }
        }
        self.stdin_enabled = false;
    }
}

impl Drop for Acia {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

impl Device for Acia {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0 => self.input.write().unwrap().pop_front().unwrap_or(b'?'),
            // status bit 4: tx data reg empty (always in our case); bit 3: rx data reg full
            1 => {
                if self.input.read().unwrap().is_empty() {
                    0b00010000
                } else {
                    0b00011000
                }
            }
            2 => self.command,
            3 => self.control,
            _ => 0, // should not happen
        }
    }
    fn write(&mut self, addr: u16, byte: u8) {
        match addr {
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

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use tracing::info;
use std::time::Duration;

fn read_char_non_blocking() -> Option<u8> {
    // 1. Enable raw mode to read keys instantly without waiting for Enter
    if enable_raw_mode().is_err() {
        return None;
    }

    let mut pressed_char = None;

    // 2. Poll checks for an event. A timeout of 0 makes it non-blocking.
    if let Ok(true) = event::poll(Duration::from_secs(0)) {
        // 3. If an event is present, read it
        if let Ok(Event::Key(key_event)) = event::read() {
            if key_event.is_press() {
                if let KeyCode::Char(c) = key_event.code {
                    if c.is_ascii() {
                        if key_event.modifiers.bits() == 0 {
                            pressed_char = Some(c as u8);
                        } else {
                            pressed_char = Some(c as u8 - 0x60);
                        }
                    }
                }
            }
        }
    }

    // 4. Always disable raw mode before returning so the terminal behaves normally again
    let _ = disable_raw_mode();

    pressed_char
}
