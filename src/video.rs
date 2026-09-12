mod char_rom;
pub mod cia1;
pub mod control;

use std::{cell::RefCell, rc::Rc, sync::Arc};

use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow::Poll, EventLoop},
    keyboard::{KeyCode, NativeKeyCode},
    window::{WindowAttributes, WindowId},
};

use crate::{
    cpu6502::{
        cpu::CPU,
        memory::Memory,
    },
    video::{
        char_rom::CHARS,
        cia1::Cia1,
        control::{Control, ScreenMode, ScreenState},
    },
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
    ram_base: u16,
    color_ram_base: u16,
    pixels: Option<Pixels<'a>>,
    window: Option<Arc<winit::window::Window>>,
    system: VideoSystem,
    rows: u8,
    cols: u8,
    regs: Rc<RefCell<Control>>,
}

impl<'a> Video<'a> {
    const BYTES_PER_PIXEL: usize = 4;

    pub fn new(control: Rc<RefCell<Control>>, ram_base: u16, color_ram_base: u16) -> Self {
        Video {
            ram_base,
            color_ram_base,
            pixels: None,
            window: None,
            system: PAL,
            rows: 25,
            cols: 40,
            regs: control,
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
    fn do_blank_screen(&mut self, border_col: u8) {
        if let Some(pixels) = self.pixels.as_mut() {
            let frame = pixels.frame_mut();
            let scr_border_col = C64_PALETTE[border_col as usize];
            frame.copy_from_slice(&scr_border_col.repeat((frame.len()) / Self::BYTES_PER_PIXEL));
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
    y_max: 255,
    x_min: 54,
    x_max: 373,
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
    cia1: Cia1,
}

impl<'a, 'b, 'c> AppHandler<'a, 'b, 'c> {
    pub fn new(cpu: CPU<'a>, video: Video<'b>, mem: Memory<'c>, cia1: Cia1) -> Self {
        Self {
            cpu,
            video,
            mem,
            cia1,
        }
    }
    pub fn run(&mut self) {
        let ev_loop = EventLoop::new().unwrap();
        ev_loop.set_control_flow(Poll);
        ev_loop.run_app(self).unwrap();
    }
    fn redraw_screen(&mut self) {
        let regs = self.video.regs.clone();
        match (
            regs.borrow().get_screen_state(),
            regs.borrow().get_screen_mode(),
        ) {
            (ScreenState::On, ScreenMode::Text) => {
                for row in 0..self.video.rows {
                    for col in 0..self.video.cols {
                        let offset = row as u16 * self.video.cols as u16 + col as u16;
                        let ch = self.mem.load_memory_byte(self.video.ram_base + offset);
                        let fg_col = self
                            .mem
                            .load_memory_byte(self.video.color_ram_base + offset);
                        let bg_col = regs.borrow().get_background_color();
                        self.video.do_char_at(ch, col, row, fg_col, bg_col as u8);
                    }
                }
            }
            (ScreenState::On, ScreenMode::Bitmap) => self
                .video
                .do_blank_screen(regs.borrow().get_border_color()),
            (ScreenState::Off, _) => self
                .video
                .do_blank_screen(regs.borrow().get_border_color()),
        }
    }
}
impl ApplicationHandler for AppHandler<'_, '_, '_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let regs = self.video.regs.clone();
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
        self.video
            .do_blank_screen(regs.borrow().get_border_color());
        self.redraw_screen();
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
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                match key_event.physical_key {
                    winit::keyboard::PhysicalKey::Code(code) => {
                        let (row, col) = match code {
                            KeyCode::Delete => (0, 0),
                            KeyCode::Enter => (0, 1),
                            KeyCode::ArrowRight => (0, 2),
                            KeyCode::F7 => (0, 3),
                            KeyCode::F1 => (0, 4),
                            KeyCode::F3 => (0, 5),
                            KeyCode::F5 => (0, 6),
                            KeyCode::ArrowDown => (0, 7),
                            KeyCode::Digit3 => (1, 0),
                            KeyCode::KeyW => (1, 1),
                            KeyCode::KeyA => (1, 2),
                            KeyCode::Digit4 => (1, 3),
                            KeyCode::KeyZ => (1, 4),
                            KeyCode::KeyS => (1, 5),
                            KeyCode::KeyE => (1, 6),
                            KeyCode::ShiftLeft => (1, 7),
                            KeyCode::Digit5 => (2, 0),
                            KeyCode::KeyR => (2, 1),
                            KeyCode::KeyD => (2, 2),
                            KeyCode::Digit6 => (2, 3),
                            KeyCode::KeyC => (2, 4),
                            KeyCode::KeyF => (2, 5),
                            KeyCode::KeyT => (2, 6),
                            KeyCode::KeyX => (2, 7),
                            KeyCode::Digit7 => (3, 0),
                            KeyCode::KeyY => (3, 1),
                            KeyCode::KeyG => (3, 2),
                            KeyCode::Digit8 => (3, 3),
                            KeyCode::KeyB => (3, 4),
                            KeyCode::KeyH => (3, 5),
                            KeyCode::KeyU => (3, 6),
                            KeyCode::KeyV => (3, 7),
                            KeyCode::Digit9 => (4, 0),
                            KeyCode::KeyI => (4, 1),
                            KeyCode::KeyJ => (4, 2),
                            KeyCode::Digit0 => (4, 3),
                            KeyCode::KeyM => (4, 4),
                            KeyCode::KeyK => (4, 5),
                            KeyCode::KeyO => (4, 6),
                            KeyCode::KeyN => (4, 7),
                            KeyCode::NumpadAdd => (5, 0), // (+)
                            KeyCode::KeyP => (5, 1),
                            KeyCode::KeyL => (5, 2),
                            KeyCode::Period => (5, 4),
                            KeyCode::Semicolon => (5, 5), // (:)
                            KeyCode::Quote => (5, 6),     // (@)
                            KeyCode::Comma => (5, 7),
                            // KeyCode::(£) ignored
                            KeyCode::NumpadStar => (6, 1), // (*)
                            //KeyCode::Semicolon => (6, 2),  // (;)
                            KeyCode::Backquote => (6, 3),  // (HOME)
                            KeyCode::ShiftRight => (6, 4),
                            KeyCode::IntlRo => (6, 5), // (=)
                            // KeyCode::(up) ignored
                            KeyCode::Minus => (6, 7), // (/)
                            KeyCode::Digit1 => (7, 0),
                            // KeyCode::(left) ignored
                            KeyCode::ControlLeft => (7, 2),
                            KeyCode::Digit2 => (7, 3),
                            KeyCode::Space => (7, 4),
                            KeyCode::AltLeft => (7, 5), // (C=)
                            KeyCode::KeyQ => (7, 6),
                            KeyCode::Tab => (7, 7), // (Stop)
                            _ => return, // panic!("Should not happen: {:?}", code), // ignore, e.g. SuperLeft
                        };
                        self.cia1
                            .set_key_state(row, col, key_event.state == ElementState::Pressed);
                    }
                    winit::keyboard::PhysicalKey::Unidentified(NativeKeyCode::MacOS(code))
                        if code == b'^' as u16 =>
                    {
                        self.cia1
                            .set_key_state(5, 3, key_event.state == ElementState::Pressed)
                    }
                    _ => {} // ignore
                }
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.cia1.step() {
            self.cpu.request_interrupt();
        }
        self.cpu.step(&mut self.mem);
        let regs_ = self.video.regs.clone();
        let mut regs = regs_.borrow_mut();
        regs.inc_current_raster_line();
        let current_raster_line = regs.get_current_raster_line();
        if current_raster_line % 20000 == 0 {
            if let Some(w) = &mut self.video.window {
                w.request_redraw();
            }
        }
        if regs.is_raster_interrupt_enabled() && current_raster_line
            == regs.get_raster_interrupt_at_line()
        {
            regs.set_source_is_raster_interrupt();
            self.cpu.request_interrupt();
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
