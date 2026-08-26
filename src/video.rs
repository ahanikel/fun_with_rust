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

/**
 *  0400-07E7 Default screen memory
 *  07E8-07F7 unused
 *  07F8-07FF Default area for sprite pointers
 *  D000-D3FF Video display control registers
 *  D800-DBE7 Color RAM (1000 bytes)
 *  DBE8-DBFF unused
 */
#[derive(Default)]
pub struct Video {}

impl Video {
    pub fn run(&mut self) {
        let ev_loop = EventLoop::new().unwrap();
        ev_loop.set_control_flow(Poll);
        let mut screen = Screen::default();
        ev_loop.run_app(&mut screen).unwrap();
    }
}

#[derive(Default)]
pub struct Screen<'a> {
    pixels: Option<Pixels<'a>>,
}

const WIDTH: u32 = 403;
const HEIGHT: u32 = 284;

impl ApplicationHandler for Screen<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new({
            let size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 * 3.0);
            let attr = WindowAttributes::default()
                .with_title("Commodore 64")
                .with_inner_size(size);
            event_loop.create_window(attr).unwrap()
        });
        self.pixels = {
            let size = window.inner_size();
            let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
            Some(
                PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
                    .build()
                    .unwrap(),
            )
        };
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            // TODO: only redraw if needed
            WindowEvent::RedrawRequested => {
                if let Some(pixels) = self.pixels.as_mut() {
                    let frame = pixels.frame_mut();
                    for (_i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                        //let x = (i % WIDTH as usize) as u8;
                        //let y = (i / WIDTH as usize) as u8;
                        // RGBA color assignment: C64 blue background
                        //pixel.copy_from_slice(&[0x40, 0x40, 0xe0, 0xff]);
                        pixel.copy_from_slice(&C64_PALETTE[C64Colour::Blue as usize]);
                    }
                    if let Err(err) = pixels.render() {
                        eprintln!("ERROR: Rendering failed: {err}");
                        event_loop.exit();
                    }
                }
            }

            _ => (),
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