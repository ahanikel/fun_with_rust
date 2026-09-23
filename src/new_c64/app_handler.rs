use std::sync::Arc;

use pixels::{PixelsBuilder, SurfaceTexture};
use tracing::info;
use winit::{application::ApplicationHandler, dpi::LogicalSize, event::{ElementState, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow::Poll, EventLoop}, keyboard::{KeyCode, NativeKeyCode}, window::{WindowAttributes, WindowId}};

use crate::new_c64::{cia1::Cia1, cpu::Cpu, memory::Memory, video::Video};

pub struct AppHandler<'v> {
    cpu: Cpu,
    video: Video<'v>,
    mem: Memory,
    cia1: Cia1,
}

impl<'v> AppHandler<'v> {
    pub fn new(cpu: Cpu, video: Video<'v>, mem: Memory, cia1: Cia1) -> Self {
        Self {
            cpu,
            video,
            mem,
            cia1,
        }
    }
    pub fn run(&mut self) {
        let ev_loop = EventLoop::new().expect("Unable to construct new EventLoop");
        ev_loop.set_control_flow(Poll);
        ev_loop.run_app(self).expect("An error occurred while executing the EventLoop");
    }
}

impl ApplicationHandler for AppHandler<'_> {
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
        self.video.do_blank_screen();
        self.video.redraw_screen(&self.mem, &self.cia1);
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
            WindowEvent::RedrawRequested => self.video.render_pixels(),
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
                            KeyCode::Backquote => (6, 3), // (HOME)
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
            if self.cia1.is_timer_a_underrun() {
                self.video.redraw_screen(&mut self.mem, &mut self.cia1);
                self.cpu.request_interrupt();
            }
        }
        self.cpu.step(&mut self.mem, &mut self.video, &mut self.cia1);
        if self.video.step(&self.mem, &self.cia1) {
            info!("Raster interrupt");
            self.cpu.request_interrupt();
        }
    }
}

