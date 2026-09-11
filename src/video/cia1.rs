
#![allow(unused)]
use chrono::Timelike;

use crate::cpu6502::memory::Device;
mod test;

/**
 * Implementation of the CIA1
 * Joystick, paddle, lightpen, serial, etc aren't implemented on purpose
 * The rest should be, unless I forgot (or there's a bug)
 */
pub struct Cia1 {
    pub port_a: u8, // write to select which keyboard row to read
    pub port_b: u8, // read column from selected row
    pub ddr_a: u8,
    pub ddr_b: u8,
    // Keyboard matrix (8x8):
    // each byte represents a row, each bit represents a column, 
    // 0 = pressed, 1 = released
    pub matrix: [u8; 8],
    pub serial_shift: u8,
    pub timer_a_ctrl: u8,
    pub timer_b_ctrl: u8,
    pub interrupt_ctrl: u8,
    pub interrupt_status: u8,
    pub timer_a_value: u16,
    pub timer_a_latch: u16,
    pub timer_b_value: u16,
    pub timer_b_latch: u16,
    pub alarm_triggered: bool,
    pub alarm_time: [u8;4],
    pub current_time: [u8;4],
}

impl Cia1 {
    pub fn new() -> Self {
        Self {
            port_a: 0xff,
            port_b: 0xff,
            ddr_a: 0,
            ddr_b: 0,
            matrix: [0xff; 8],
            serial_shift: 0,
            timer_a_ctrl: 0,
            timer_b_ctrl: 0,
            interrupt_ctrl: 0,
            interrupt_status: 0,
            timer_a_value: 0,
            timer_a_latch: 0,
            timer_b_value: 0,
            timer_b_latch: 0,
            alarm_triggered: false,
            alarm_time: [0;4],
            current_time: read_current_time(),
        }
    }
    pub fn step(&mut self) -> bool {
        let mut interrupt = false;
        self.current_time = read_current_time();
        if self.is_timer_a_started() {
            if self.timer_a_value == 0 {
                if self.is_timer_a_stopping() {
                    self.set_timer_a_started(false);
                } else {
                    self.timer_a_value = self.timer_a_latch;
                }
                self.signal_timer_a_underrun();
                if self.is_timer_a_int_enabled() {
                    interrupt = true;
                }
            } else {
                self.timer_a_value -= 1;
            }
        }
        if self.is_timer_b_started() {
            if self.timer_b_value == 0 {
                if self.is_timer_b_stopping() {
                    self.set_timer_b_started(false);
                } else {
                    self.timer_b_value = self.timer_b_latch;
                }
                self.signal_timer_b_underrun();
                if self.is_timer_b_int_enabled() {
                    interrupt = true;
                }
            } else {
                self.timer_b_value -= 1;
            }
        }
        if self.is_timer_a_load_once() {
            self.timer_a_value = self.timer_a_latch;
            self.set_timer_a_load_once(false);
        }
        if self.is_timer_b_load_once() {
            self.timer_b_value = self.timer_b_latch;
            self.set_timer_b_load_once(false);
        }
        let now = [self.read(0x8),
        self.read(0x9),
        self.read(0xa),
        self.read(0xb)];
        if self.is_alarm_enabled() && !self.alarm_triggered && time_has_reached_alarm(self.current_time, self.alarm_time) {
            interrupt = true;
            self.set_alarm_triggered();
        }
        interrupt
    }
    pub fn set_key_state(&mut self, row: usize, col: usize, is_pressed: bool) {
        if is_pressed {
            self.matrix[row] &= !(1 << col);
        } else {
            self.matrix[row] |= 1 << col;
        }
    }
    pub fn is_timer_a_started(&self) -> bool {
        self.timer_a_ctrl & 1 != 0
    }
    pub fn set_timer_a_started(&mut self, value: bool) {
        if value {
            self.timer_a_ctrl |= 1;
        } else {
            self.timer_a_ctrl &= !1;
        }
    }
    pub fn is_timer_b_started(&self) -> bool {
        self.timer_b_ctrl & 1 != 0
    }
    pub fn set_timer_b_started(&mut self, value: bool) {
        if value {
            self.timer_b_ctrl |= 1;
        } else {
            self.timer_b_ctrl &= !1;
        }
    }
    pub fn is_timer_a_changing_port_b(&self) -> bool {
        self.timer_a_ctrl & 2 != 0
    }
    pub fn is_timer_b_changing_port_b(&self) -> bool {
        self.timer_b_ctrl & 2 != 0
    }
    pub fn is_timer_a_inverting_port_b(&self) -> bool {
        self.timer_a_ctrl & 4 == 0
    }
    pub fn is_timer_b_inverting_port_b(&self) -> bool {
        self.timer_b_ctrl & 4 == 0
    }
    pub fn is_timer_a_stopping(&self) -> bool {
        self.timer_a_ctrl & 8 != 0
    }
    pub fn is_timer_b_stopping(&self) -> bool {
        self.timer_b_ctrl & 8 != 0
    }
    pub fn is_timer_a_load_once(&self) -> bool {
        self.timer_a_ctrl & 16 != 0
    }
    pub fn set_timer_a_load_once(&mut self, value: bool) {
        if value {
            self.timer_a_ctrl |= 16;
        } else {
            self.timer_a_ctrl &= !16;
        }
    }
    pub fn is_timer_b_load_once(&self) -> bool {
        self.timer_b_ctrl & 16 != 0
    }
    pub fn set_timer_b_load_once(&mut self, value: bool) {
        if value {
            self.timer_b_ctrl |= 16;
        } else {
            self.timer_b_ctrl &= !16;
        }
    }
    pub fn is_timer_a_underrun(&self) -> bool {
        self.interrupt_status & 1 != 0
    }
    pub fn is_timer_b_underrun(&self) -> bool {
        self.interrupt_status & 2 != 0
    }
    pub fn signal_timer_a_underrun(&mut self) {
        self.interrupt_status |= 1;
    }
    pub fn signal_timer_b_underrun(&mut self) {
        self.interrupt_status |= 2;
    }
    pub fn is_timer_a_int_enabled(&self) -> bool {
        self.interrupt_ctrl & 1 != 0
    }
    pub fn is_timer_b_int_enabled(&self) -> bool {
        self.interrupt_ctrl & 2 != 0
    }
    pub fn is_writing_alarm(&self) -> bool {
        self.timer_b_ctrl & 0x80 != 0
    }
    pub fn is_alarm_enabled(&self) -> bool {
        self.interrupt_ctrl & 4 != 0
    }
    pub fn set_alarm_triggered(&mut self) {
        self.alarm_triggered = true;
        self.interrupt_status |= 4;
    }
}

impl Device for Cia1 {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // port_a: output pins reflect port_a, input pins float "high"
            0x0 => (self.port_a & self.ddr_a) | !self.ddr_a,
            // port_b: matrix scan logic
            0x1 => {
                let mut res = 0xff;
                let row_sel = (self.port_a & self.ddr_a) | !self.ddr_a;
                for row in 0..8 {
                    // active low
                    if (row_sel & (1 << row)) == 0 {
                        res &= self.matrix[row];
                    }
                }
                (res & !self.ddr_b) | (self.port_b & self.ddr_b)
            }
            0x2 => self.ddr_a,
            0x3 => self.ddr_b,
            0x4 => self.timer_a_value as u8,
            0x5 => (self.timer_a_value >> 8) as u8,
            0x6 => self.timer_b_value as u8,
            0x7 => (self.timer_b_value >> 8) as u8,
            0x8 => self.current_time[0],
            0x9 => self.current_time[1],
            0xa => self.current_time[2],
            0xb => self.current_time[3],
            0xc => self.serial_shift,
            0xd => self.interrupt_status,
            0xe => self.timer_a_ctrl,
            0xf => self.timer_b_ctrl,
            _ if addr >= 0x10 && addr <= 0xff => self.read(addr % 0x10),
            _ => panic!("Should not happen (misconfiguration)"),
        }
    }

    fn write(&mut self, addr: u16, byte: u8) {
        match addr {
            0x0 => self.port_a = byte,
            0x1 => self.port_b = byte,
            0x2 => self.ddr_a = byte,
            0x3 => self.ddr_b = byte,
            0x4 => {
                self.timer_a_value &= 0xff00;
                self.timer_a_value |= byte as u16;
                self.timer_a_latch &= 0xff00;
                self.timer_a_latch |= byte as u16;
            }
            0x5 => {
                self.timer_a_latch &= 0x00ff;
                self.timer_a_latch |= (byte as u16) << 8;
                if !self.is_timer_a_started() {
                    self.timer_a_value &= 0x00ff;
                    self.timer_a_value |= (byte as u16) << 8;
                }
            }
            0x6 => {
                self.timer_b_value &= 0xff00;
                self.timer_b_value |= byte as u16;
                self.timer_b_latch &= 0xff00;
                self.timer_b_latch |= byte as u16;
            }
            0x7 => {
                self.timer_b_latch &= 0x00ff;
                self.timer_b_latch |= (byte as u16) << 8;
                if !self.is_timer_b_started() {
                    self.timer_b_value &= 0x00ff;
                    self.timer_b_value |= (byte as u16) << 8;
                }
            }
            0x8 => if self.is_writing_alarm() { self.alarm_time[0] = byte; },
            0x9 => if self.is_writing_alarm() { self.alarm_time[1] = byte; },
            0xa => if self.is_writing_alarm() { self.alarm_time[2] = byte; },
            0xb => if self.is_writing_alarm() { self.alarm_time[3] = byte; },
            0xc => self.serial_shift = byte,
            0xd => {
                if byte & 0x80 != 0 {
                    self.interrupt_ctrl |= byte;
                } else {
                    self.interrupt_ctrl &= !byte;
                }
            }
            0xe => self.timer_a_ctrl = byte,
            0xf => self.timer_b_ctrl = byte,
            _ if addr >= 0x10 && addr <= 0xff => self.write(addr % 0x10, byte),
            _ => panic!("Should not happen (misconfiguration)"),
        }
    }
}

pub fn to_bcd(num: u8) -> u8 {
    assert!(num < 100);
    let msb = num / 10;
    let lsb = num - msb * 10;
    msb * 16 + lsb
}

pub fn time_has_reached_alarm(time: [u8;4], alarm: [u8;4]) -> bool {
    u32::from_le_bytes(time) >= u32::from_le_bytes(alarm)
}

pub fn read_current_time() -> [u8;4] {
    let time = chrono::Local::now();
    [
        (time.nanosecond() / 100_000_000) as u8,
        to_bcd(time.second() as u8),
        to_bcd(time.minute() as u8),
        {
            let (pm, hour ) = time.hour12();
            to_bcd(hour as u8) | if pm { 0x80 } else { 0x00 }
        }
    ]
}