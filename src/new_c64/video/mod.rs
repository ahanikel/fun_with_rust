pub mod control;

use std::sync::Arc;

use pixels::Pixels ;
use tracing::warn;

use crate::new_c64::{cia1::Cia1, memory::Memory, video::control::{Control, ScreenMode, ScreenState}};

/**
 *  0400-07E7 Default screen memory
 *  07E8-07F7 unused
 *  07F8-07FF Default area for sprite pointers
 *  D000-D3FF Video display control registers
 *  D800-DBE7 Color RAM (1000 bytes)
 *  DBE8-DBFF unused
 */
pub struct Video<'a> {
    pub(crate) ram_base: u16,
    pub(crate) color_ram_base: u16,
    pub(crate) pixels: Option<Pixels<'a>>,
    pub(crate) window: Option<Arc<winit::window::Window>>,
    pub(crate) system: VideoSystem,
    pub(crate) rows: u8,
    pub(crate) cols: u8,
    regs: Control,
    pseudo_pixel_counter: usize,
}

impl Default for Video<'_> {
    fn default() -> Self {
        Self::new(Control::new(), 0x0400, 0xd800)
    }
}
impl<'a> Video<'a> {
    const BYTES_PER_PIXEL: usize = 4;

    pub fn new(control: Control, ram_base: u16, color_ram_base: u16) -> Self {
        Video {
            ram_base,
            color_ram_base,
            pixels: None,
            window: None,
            system: PAL,
            rows: 25,
            cols: 40,
            regs: control,
            pseudo_pixel_counter: 0,
        }
    }

    #[allow(unused)]
    pub fn do_empty_screen(&mut self, border_col: u8, bg_col: u8) {
        if let Some(pixels) = self.pixels.as_mut() {
            let frame = pixels.frame_mut();
            let scr_border_col = C64_PALETTE[border_col as usize];
            let scr_bg_col = C64_PALETTE[bg_col as usize];
            let frame_pixel_factor = Self::BYTES_PER_PIXEL;
            let frame_line_width = self.system.width * frame_pixel_factor;

            // top border
            let line = &mut frame[0..self.system.y_min * frame_line_width];
            line.copy_from_slice(&scr_border_col.repeat((line.len()) / Self::BYTES_PER_PIXEL));

            // bottom border
            let line = &mut frame[self.system.y_max * frame_line_width..];
            line.copy_from_slice(&scr_border_col.repeat((line.len()) / Self::BYTES_PER_PIXEL));

            // left / right borders next to content area
            for line_no in self.system.y_min..self.system.y_max {
                // left border
                let from = line_no * frame_line_width;
                let to = from + self.system.x_min * frame_pixel_factor;
                let (frame1, frame2) = frame.split_at_mut(to);
                let line = &mut frame1[from..to];
                line.copy_from_slice(&scr_border_col.repeat((line.len()) / Self::BYTES_PER_PIXEL));
                // right border
                let from2 = from + (self.system.x_max + 1) * frame_pixel_factor - to;
                let to2 = (line_no + 1) * frame_line_width - to;
                let (frame3, frame2) = frame2.split_at_mut(from2);
                let line2 = &mut frame2[0..to2 - from2];
                line2
                    .copy_from_slice(&scr_border_col.repeat((line2.len()) / Self::BYTES_PER_PIXEL));
                // content area
                frame3.copy_from_slice(&scr_bg_col.repeat((frame3.len()) / Self::BYTES_PER_PIXEL));
            }
        }
    }
    pub fn do_char_at(&mut self, mem: &Memory, cia1: &Cia1, ch: u8, x: u8, y: u8, fg_col: u8, bg_col: u8) {
        if self.pixels.is_some() {
            let frame_line_width = self.system.width * Self::BYTES_PER_PIXEL;
            let scan_y = self.system.y_min * frame_line_width;
            for char_line in 0..8 {
                let scan_y_offset = (y as usize * 8 + char_line) * frame_line_width;
                let pixels = mem.read(self, cia1, 0xd000 + (ch as u16 * 8 + char_line as u16));
                let scan_x_offset = (self.system.x_min + x as usize * 8) * Self::BYTES_PER_PIXEL;
                let frame = self.pixels.as_mut().unwrap().frame_mut();
                for bit in 0..8 {
                    // bit 7 is the leftmost pixel on the screen
                    let is_set = pixels & (0x80 >> bit) != 0;
                    let color = if is_set {
                        C64_PALETTE[fg_col as usize]
                    } else {
                        C64_PALETTE[bg_col as usize]
                    };
                    let pos = scan_y + scan_y_offset + scan_x_offset + bit * Self::BYTES_PER_PIXEL;
                    frame[pos..pos + Self::BYTES_PER_PIXEL].copy_from_slice(&color);
                }
            }
        }
    }
    pub fn do_blank_screen(&mut self) {
        let border_col = self.regs.get_border_color();
        if let Some(pixels) = self.pixels.as_mut() {
            let frame = pixels.frame_mut();
            let scr_border_col = C64_PALETTE[border_col as usize];
            frame.copy_from_slice(&scr_border_col.repeat((frame.len()) / Self::BYTES_PER_PIXEL));
        }
    }
    pub fn redraw_screen(&mut self, mem: &Memory, cia1: &Cia1) {
        let (state, mode) = (
            self.regs.get_screen_state(),
            self.regs.get_screen_mode(),
        );
        match (state, mode) {
            (ScreenState::On, ScreenMode::Text) => {
                for row in 0..self.rows {
                    for col in 0..self.cols {
                        let offset = row as u16 * self.cols as u16 + col as u16;
                        let ch = mem.read(self, cia1, self.ram_base + offset);
                        let fg_col = mem.read(self, cia1, self.color_ram_base + offset);
                        let bg_col = self.regs.get_background_color();
                        self.do_char_at(mem, cia1, ch, col, row, fg_col, bg_col as u8);
                    }
                }
            }
            (ScreenState::On, ScreenMode::Bitmap) => {self.do_blank_screen()},
            (ScreenState::Off, _) => self.do_blank_screen(),
        }
    }

    pub fn render_pixels(&self) {
        if let Some(p) = &self.pixels {
            p.render().unwrap_or(warn!("Failed to render pixels"))
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.regs.read(addr)
    }

    pub fn write(&mut self, addr: u16, byte: u8) {
        self.regs.write(addr, byte);
    }

    /*
        Returns true if interrupt should be triggered
     */
    pub fn step(&mut self, mem: &Memory, cia1: &Cia1) -> bool {
        self.pseudo_pixel_counter = self.pseudo_pixel_counter.wrapping_add(1);
        if self.pseudo_pixel_counter % self.system.x_max == 0 {
            self.inc_current_raster_line();
        }

        if self.pseudo_pixel_counter % 20000 == 0 {
            self.redraw_screen(mem, cia1);
            if let Some(w) = &mut self.window {
                w.request_redraw();
            }
        }

        if self.regs.is_raster_interrupt_enabled()
            && self.regs.get_current_raster_line() == self.regs.get_raster_interrupt_at_line()
        {
            self.regs.set_source_is_raster_interrupt();
            true
        } else {
            false
        }
    }

    pub fn inc_current_raster_line(&mut self) {
        self.regs.inc_current_raster_line();
    }
}

pub struct VideoSystem {
    pub width: usize,
    pub height: usize,
    pub y_min: usize,
    pub y_max: usize,
    pub x_min: usize,
    pub x_max: usize,
}

#[allow(unused)]
pub const PAL: VideoSystem = VideoSystem {
    width: 428,
    height: 312,
    y_min: 28,
    y_max: 255,
    x_min: 54,
    x_max: 373,
};
#[allow(unused)]
pub const NTSC: VideoSystem = VideoSystem {
    width: 384,
    height: 272,
    y_min: 36,
    y_max: 235,
    x_min: 32,
    x_max: 351,
};

// C64 colour indices (0-15)
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum C64Colour {
    Black = 0,
    White = 1,
    Red = 2,
    Cyan = 3,
    Purple = 4,
    Green = 5,
    Blue = 6,
    Yellow = 7,
    Orange = 8,
    Brown = 9,
    LightRed = 10,
    DarkGrey = 11,
    MediumGrey = 12,
    LightGreen = 13,
    LightBlue = 14,
    LightGrey = 15,
}

// C64 colour palette (RGB + Alpha)
pub const C64_PALETTE: [[u8; 4]; 16] = [
    [0x00, 0x00, 0x00, 0xFF], // 0: Black
    [0xFF, 0xFF, 0xFF, 0xFF], // 1: White
    [0x88, 0x39, 0x32, 0xFF], // 2: Red
    [0x67, 0xB6, 0xBD, 0xFF], // 3: Cyan
    [0x8B, 0x3F, 0x96, 0xFF], // 4: Purple
    [0x55, 0xA0, 0x49, 0xFF], // 5: Green
    [0x40, 0x31, 0x8D, 0xFF], // 6: Blue
    [0xBF, 0xCE, 0x72, 0xFF], // 7: Yellow
    [0x8B, 0x54, 0x29, 0xFF], // 8: Orange
    [0x57, 0x42, 0x00, 0xFF], // 9: Brown
    [0xB8, 0x69, 0x62, 0xFF], // 10: Light Red
    [0x50, 0x50, 0x50, 0xFF], // 11: Dark Grey
    [0x7A, 0x7A, 0x7A, 0xFF], // 12: Medium Grey
    [0x94, 0xE0, 0x89, 0xFF], // 13: Light Green
    [0x78, 0x6F, 0xC6, 0xFF], // 14: Light Blue
    [0x9F, 0x9F, 0x9F, 0xFF], // 15: Light Grey
];
