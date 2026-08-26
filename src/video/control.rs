#![allow(dead_code)]
struct Control<'a> {
    reg: &'a [u8; 0x40],
    raster_interrupt_at_line: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Coordinate {
    X = 0,
    Y = 1,
}

#[derive(Debug, Clone, Copy)]
enum ScreenState {
    Off, On
}

#[derive(Debug, Clone, Copy)]
enum ScreenMode {
    Text, Bitmap
}

impl Control<'_> {
    fn get_sprite_coordinate(&self, sprite_no: u8, coordinate: Coordinate) -> u16 {
        debug_assert!(sprite_no < 8);
        let lsb = self.reg[sprite_no as usize * 2 + coordinate as usize] as u16;
        let msb = if coordinate == Coordinate::X && self.reg[0x10] & 1 << sprite_no != 0 {
            1 << 8
        } else {
            0
        };
        lsb + msb
    }
    fn get_vertical_raster_scroll(&self) -> u8 {
        self.reg[0x11] & 0b111
    }
    fn get_screen_height(&self) -> u8 {
        if self.reg[0x11] & 0b1000 == 0 { 24 } else { 25 }
    }
    fn get_screen_state(&self) -> ScreenState {
        if self.reg[0x11] & 0b10000 == 0 { ScreenState::Off } else { ScreenState::On }
    }
    fn get_screen_mode(&self) -> ScreenMode {
        if self.reg[0x11] & 0b100000 == 0 { ScreenMode::Text } else { ScreenMode::Bitmap }
    }
    fn get_extended_background_mode(&self) -> bool {
        self.reg[0x11] & 0b1000000 != 0
    }
    fn get_current_raster_line(&self) -> u16 {
        let lsb = self.reg[0x12] as u16; 
        let msb = if self.reg[0x11] & 0x80 != 0 { 256 } else { 0 };
        lsb + msb
    }
    fn set_raster_interrupt_at_line(&mut self, line: u16) {
        self.raster_interrupt_at_line = line;
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
        let c = Control { reg: &reg, raster_interrupt_at_line: 0 };
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
        let c = Control { reg: &reg, raster_interrupt_at_line: 0 };
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
