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

use crate::{cpu6502::{cpu::CPU, memory::Memory}, video::char_rom::CHARS};

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
            self.do_char_line(
                mem,
                self.line,
                C64Colour::LightBlue as u8,
                C64Colour::Blue as u8,
            );
            self.line += 1;
        } else {
            self.line = 0;
        }
    }
    fn do_char_line(&mut self, mem: &mut Memory, scan_y: usize, border_col: u8, bg_col: u8) {
        if let Some(pixels) = self.pixels.as_mut() {
            let frame = pixels.frame_mut();
            let line_start = self.line * self.system.width * 4;
            let line = &mut frame[line_start..line_start + self.system.width * 4];
            let scr_border_col = C64_PALETTE[border_col as usize];
            if scan_y < self.system.y_min || scan_y > self.system.y_max {
                line.copy_from_slice(&scr_border_col.repeat((line.len()) / 4));
                return;
            }
            line[..self.system.y_min].fill(border_col);
            line[self.system.y_max + 1..].fill(border_col);
            let inner_y = scan_y - self.system.y_min;
            let char_row = inner_y / 8; // 0..24
            let char_line = inner_y % 8; // line within character
            for col in 0..40 {
                let char_idx = char_row * 40 + col;
                // TODO: this seems correct but not performant
                let char_code = mem.load_memory_byte(self.ram_base + char_idx as u16) as usize;
                let fg_col = mem.load_memory_byte(self.color_ram_base + char_idx as u16);
                let pixels = CHARS[char_code * 8 + char_line];
                let scan_x = self.system.y_min + col * 8;
                for bit in 0..8 {
                    // bit 7 is the leftmost pixel on the screen
                    let is_set = pixels & (0x80 >> bit) != 0;
                    let color = if is_set {
                        C64_PALETTE[fg_col as usize]
                    } else {
                        C64_PALETTE[bg_col as usize]
                    };
                    let pos = scan_x + bit;
                    line[pos..pos + 4].copy_from_slice(&color);
                }
            }
            pixels.render().unwrap();
        }
    }
}

struct VideoSystem {
    width: usize,
    height: usize,
    y_min: usize,
    y_max: usize,
}

#[allow(unused)]
const PAL: VideoSystem = VideoSystem {
    width: 403,
    height: 284,
    y_min: 51,
    y_max: 250,
};
#[allow(unused)]
const NTSC: VideoSystem = VideoSystem {
    width: 384,
    height: 272,
    y_min: 36,
    y_max: 235,
};

pub struct AppHandler<'a,'b,'c> {
    cpu: CPU<'a>,
    video: Video<'b>,
    mem: Memory<'c>,
}

impl <'a,'b,'c> AppHandler<'a,'b,'c> {
    pub fn new(cpu: CPU<'a>, video: Video<'b>, mem: Memory<'c>) -> Self {
        Self { cpu, video, mem }
    }
    pub fn run(&mut self) {
        let ev_loop = EventLoop::new().unwrap();
        ev_loop.set_control_flow(Poll);
        ev_loop.run_app(self).unwrap();
    }
}
impl ApplicationHandler for AppHandler<'_,'_,'_> {
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
            WindowEvent::RedrawRequested => {}
            _ => (),
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.cpu.step(&mut self.mem);
        self.video.step(&mut self.mem);
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
