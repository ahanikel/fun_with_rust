#![allow(dead_code)]

use std::ops::Range;

use crate::{cpu6502::memory::Device, video::C64Colour};
pub struct Control {
    pub reg: [u8; 0x40],
    pub raster_interrupt_at_line: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Coordinate {
    X = 0,
    Y = 1,
}

impl From<u8> for Coordinate {
    fn from(value: u8) -> Self {
        if value == 0 {
            Coordinate::X
        } else {
            Coordinate::Y
        }
    }
}

impl From<u16> for Coordinate {
    fn from(value: u16) -> Self {
        if value == 0 {
            Coordinate::X
        } else {
            Coordinate::Y
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScreenState {
    Off, On
}

#[derive(Debug, Clone, Copy)]
pub enum ScreenMode {
    Text, Bitmap
}

impl Control {
    pub fn new() -> Self {
        let mut reg = [0; 0x40];
        reg[0x11] = 0x1b;
        reg[0x16] = 0xc8;
        reg[0x18] = 0x14;
        let mut ret = Control { reg, raster_interrupt_at_line: 0 };
        ret.set_border_color(C64Colour::LightBlue as u8);
        ret.set_background_color(C64Colour::Blue as u8);
        ret
    }
    pub fn get_sprite_coordinate(&self, sprite_no: u8, coordinate_axis: Coordinate) -> u16 {
        debug_assert!(sprite_no < 8);
        let lsb = self.reg[sprite_no as usize * 2 + coordinate_axis as usize] as u16;
        let msb = if coordinate_axis == Coordinate::X && self.reg[0x10] & (1 << sprite_no) != 0 {
            1 << 8
        } else {
            0
        };
        lsb | msb
    }
    pub fn set_sprite_coordinate(&mut self, sprite_no: u8, coordinate_axis: Coordinate, value: u16) {
        self.reg[sprite_no as usize * 2 + coordinate_axis as usize] = (value & 0xff) as u8;
        if coordinate_axis == Coordinate::X && value & 0x100 != 0 {
            self.reg[0x10] |= 1 << sprite_no;
        }
    }
    pub fn get_vertical_raster_scroll(&self) -> u8 {
        self.reg[0x11] & 0b111
    }
    pub fn set_vertical_raster_scroll(&mut self, value: u8) {
        self.reg[0x11] |= value & 0b111;
    }
    pub fn get_screen_height_range(&self) -> Range<u8> {
        if self.reg[0x11] & 0b1000 == 0 { 0..24 } else { 0..25 }
    }
    pub fn set_screen_height_range(&mut self, reduced: bool) {
        if reduced { self.reg[0x11] &= !0b1000 } else { self.reg[0x11] |= 0b1000 }
    }
    pub fn get_screen_state(&self) -> ScreenState {
        if self.reg[0x11] & 0b10000 == 0 { ScreenState::Off } else { ScreenState::On }
    }
    pub fn set_screen_state(&mut self, st: ScreenState) {
        match st {
            ScreenState::Off => self.reg[0x11] &= !0b10000,
            ScreenState::On  => self.reg[0x11] |=  0b10000,
        }
    }
    pub fn get_screen_mode(&self) -> ScreenMode {
        if self.reg[0x11] & 0b100000 == 0 { ScreenMode::Text } else { ScreenMode::Bitmap }
    }
    pub fn set_screen_mode(&mut self, m: ScreenMode) {
        match m {
            ScreenMode::Text => self.reg[0x11] &=  !0b100000,
            ScreenMode::Bitmap => self.reg[0x11] |= 0b100000,
        }
    }
    pub fn get_extended_background_mode(&self) -> bool {
        self.reg[0x11] & 0b1000000 != 0
    }
    pub fn set_extended_background_mode(&mut self, extended: bool) {
        if extended { self.reg[0x11] |= 0b1000000 } else { self.reg[0x11] &= !0b1000000 }
    }
    pub fn get_current_raster_line(&self) -> u16 {
        let lsb = self.reg[0x12] as u16; 
        let msb = if self.reg[0x11] & 0x80 != 0 { 256 } else { 0 };
        lsb + msb
    }
    pub fn inc_current_raster_line(&mut self) {
        self.reg[0x12] = self.reg[0x12].wrapping_add(1);
        if self.reg[0x12] == 0 {
            self.reg[0x11] ^= 0x80;
        }
    }
    pub fn get_raster_interrupt_at_line(&self) -> u16 {
        self.raster_interrupt_at_line
    }
    pub fn set_raster_interrupt_at_line(&mut self, line: u16) {
        self.raster_interrupt_at_line = line;
    }
    pub fn get_horizontal_raster_scroll(&self) -> u8 {
        self.reg[0x16] & 0b111
    }
    pub fn set_horizontal_raster_scroll(&mut self, value: u8) {
        self.reg[0x16] |= value & 0b111;
    }
    pub fn is_multi_color_mode(&self) -> bool {
        self.reg[0x16] & 0b1000 != 0
    }
    pub fn set_multi_color_mode(&mut self, value: bool) {
        if value {
            self.reg[0x16] |=  0b1000;
        } else {
            self.reg[0x16] &= !0b1000;
        }
    }
    pub fn get_screen_width_range(&self) -> Range<u8> {
        if self.reg[0x16] & 0b1000 == 0 { 1..39 } else { 0..40 }
    }
    pub fn set_screen_width_range(&mut self, reduced: bool) {
        if reduced {
            self.reg[0x16] &= !0b1000;
        } else {
            self.reg[0x16] |= 0b1000;
        }
    }
    pub fn get_screen_base_address(&self) -> u16 {
        (self.reg[0x18] & 0xf0 >> 4) as u16 * 0x400
    }
    pub fn get_character_set_base_address(&self) -> u16 {
        (self.reg[0x18] & 0x0e >> 1) as u16 * 0x800
    }
    pub fn get_bitmap_base_address(&self) -> u16 {
        // Bit 3 selects 0x0000 or 0x2000
        (self.reg[0x18] & 0x08) as u16 * 0x400
    }
    pub fn is_raster_interrupt_enabled(&self) -> bool {
        self.reg[0x1a] & 1 != 0
    }
    pub fn set_source_is_raster_interrupt(&mut self) {
        self.reg[0x19] |= 1;
    }
    pub fn get_border_color(&self) -> u8 {
        self.reg[0x20] & 0x0f
    }
    pub fn set_border_color(&mut self, color: u8) {
        self.reg[0x20] = color & 0x0f;
    }
    pub fn get_background_color(&self) -> u8 {
        self.reg[0x21] & 0x0f
    }
    pub fn set_background_color(&mut self, color: u8) {
        self.reg[0x21] = color & 0x0f;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_get_sprite_coordinate_1() {
        let reg: [u8; 64] = [
            0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let c = Control { reg: reg, raster_interrupt_at_line: 0 };
        for sprite_no in 0..8 {
            assert_eq!(
                sprite_no as u16 + 256,
                c.get_sprite_coordinate(sprite_no, Coordinate::X)
            );
            assert_eq!(
                sprite_no as u16,
                c.get_sprite_coordinate(sprite_no, Coordinate::Y)
            );
        }
    }
    #[test]
    fn test_get_sprite_coordinate_2() {
        let reg: [u8; 64] = [
            0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 0xaa, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let c = Control { reg: reg, raster_interrupt_at_line: 0 };
        for sprite_no in 0..8_u8 {
            assert_eq!(
                sprite_no as u16 + if sprite_no.is_multiple_of(2) { 0 } else { 256 },
                c.get_sprite_coordinate(sprite_no, Coordinate::X)
            );
            assert_eq!(
                sprite_no as u16,
                c.get_sprite_coordinate(sprite_no, Coordinate::Y)
            );
        }
    }
}

impl Device for Control {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x18 => self.reg[0x18] | 1,
            0x19 => {
                let res = self.reg[0x19] & 0x0f;
                if res != 0 { res | 0x80 } else { res }
            }
            _ => self.reg[addr as usize],
        }
    }

    fn write(&mut self, addr: u16, byte: u8) {
        match addr {
            0x11 => {
                // set bits 0-6 to bits 0-6 of byte, leave bit 7 unchanged
                self.reg[0x11] = (self.reg[0x11] & 0x80) | (byte & 0x7f);
                // set bit 8 to bit 7 of byte, leave bits 0-7 unchanged
                self.raster_interrupt_at_line =
                    (self.raster_interrupt_at_line & 0xff) | ((byte as u16 & 0x80) << 1);
            }
            0x12 => {
                self.raster_interrupt_at_line =
                    (self.raster_interrupt_at_line & 0x100) | byte as u16
            }
            0x18 => {}, // we don't currently support changing the offsets
            0x19 => {
                // acknowledge interrupts
                self.reg[0x19] &= !(byte & 0x0f);
            }
            _ if addr <= 0x3f => self.reg[addr as usize] = byte,
            _ => self.write(addr % 0x40, byte),
        }
    }
}
