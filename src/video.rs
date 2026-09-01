mod char_rom;
mod control;

use std::sync::Arc;

use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow::Poll, EventLoop},
    window::{WindowAttributes, WindowId},
};

use crate::{
    cpu6502::{cpu::CPU, memory::Memory},
    video::char_rom::CHARS,
};

/**
 *  0400-07E7 Default screen memory
 *  07E8-07F7 unused
 *  07F8-07FF Default area for sprite pointers
 *  D000-D3FF Video display control registers
 *  D800-DBE7 Color RAM (1000 bytes)
 *  DBE8-DBFF unused
 */
pub struct Video<'a> {
    line: usize,
    ram_base: u16,
    color_ram_base: u16,
    pixels: Option<Pixels<'a>>,
    window: Option<Arc<winit::window::Window>>,
    system: VideoSystem,
}

impl<'a> Video<'a> {
    const BYTES_PER_PIXEL: usize = 4;

    pub fn new(ram_base: u16, color_ram_base: u16) -> Self {
        Video {
            line: 0,
            ram_base,
            color_ram_base,
            pixels: None,
            window: None,
            system: PAL,
        }
    }
    pub fn step(&mut self, mem: &mut Memory) {
        if self.line < self.system.height {
            self.line += 1;
        } else {
            self.line = 0;
        }
    }

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
    fn do_char_at(&mut self, ch: u8, x: u8, y: u8, fg_col: u8, bg_col: u8) {
        if let Some(pixels) = self.pixels.as_mut() {
            let frame = pixels.frame_mut();
            let frame_line_width = self.system.width * Self::BYTES_PER_PIXEL;
            let scan_y = self.system.y_min * frame_line_width;
            for char_line in 0..8 {
                let scan_y_offset = (y as usize * 8 + char_line) * frame_line_width;
                let pixels = CHARS[ch as usize * 8 + char_line];
                let scan_x_offset = (self.system.x_min + x as usize * 8) * Self::BYTES_PER_PIXEL;
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
}

struct VideoSystem {
    width: usize,
    height: usize,
    y_min: usize,
    y_max: usize,
    x_min: usize,
    x_max: usize,
}

#[allow(unused)]
const PAL: VideoSystem = VideoSystem {
    width: 428,
    height: 312,
    y_min: 28,
    y_max: 284,
    x_min: 25,
    x_max: 402,
};
#[allow(unused)]
const NTSC: VideoSystem = VideoSystem {
    width: 384,
    height: 272,
    y_min: 36,
    y_max: 235,
    x_min: 32,
    x_max: 351,
};

pub struct AppHandler<'a, 'b, 'c> {
    cpu: CPU<'a>,
    video: Video<'b>,
    mem: Memory<'c>,
}

impl<'a, 'b, 'c> AppHandler<'a, 'b, 'c> {
    pub fn new(cpu: CPU<'a>, video: Video<'b>, mem: Memory<'c>) -> Self {
        Self { cpu, video, mem }
    }
    pub fn run(&mut self) {
        let ev_loop = EventLoop::new().unwrap();
        ev_loop.set_control_flow(Poll);
        ev_loop.run_app(self).unwrap();
    }
}
impl ApplicationHandler for AppHandler<'_, '_, '_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new({
            let size = LogicalSize::new(
                self.video.system.width as f64 * 3.0,
                self.video.system.height as f64 * 3.0,
            );
            let attr = WindowAttributes::default()
                .with_title("Commodore 64")
                .with_inner_size(size);
            event_loop.create_window(attr).unwrap()
        });
        self.video.pixels = {
            let size = window.inner_size();
            let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
            Some(
                PixelsBuilder::new(
                    self.video.system.width as u32,
                    self.video.system.height as u32,
                    surface_texture,
                )
                .build()
                .unwrap(),
            )
        };
        window.request_redraw();
        self.video.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(p) = &mut self.video.pixels {
                    p.render().unwrap()
                }
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.cpu.step(&mut self.mem);
        self.video.step(&mut self.mem);
        self.video
            .do_empty_screen(C64Colour::LightBlue as u8, C64Colour::Blue as u8);
        self.video.do_char_at(b'H' + 0x40, 10, 10, C64Colour::White as u8, C64Colour::Red as u8);
        self.video.do_char_at(b'E' + 0x40, 11, 10, C64Colour::White as u8, C64Colour::Cyan as u8);
        self.video.do_char_at(b'L' + 0x40, 12, 10, C64Colour::White as u8, C64Colour::Purple as u8);
        self.video.do_char_at(b'L' + 0x40, 13, 10, C64Colour::White as u8, C64Colour::Green as u8);
        self.video.do_char_at(b'O' + 0x40, 14, 10, C64Colour::White as u8, C64Colour::Yellow as u8);
        if let Some(w) = &mut self.video.window {
            w.request_redraw();
        }
    }
}

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
