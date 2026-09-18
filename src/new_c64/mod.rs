#![allow(unused)]

mod char_rom;
mod memory;
mod cpu;
mod video;
mod cia1;
mod app_handler;

use std::{io::Read, path::{Path, PathBuf}};

use crate::new_c64::{app_handler::AppHandler, cia1::Cia1, cpu::Cpu, memory::Memory, video::Video};

pub fn c64(kernal: &Path, basic: &Path, verbose: bool) {
    let cpu = Cpu::new(true);
    let video = Video::default();
    let mem = Memory::new(kernal, basic);
    let cia1 = Cia1::new();
    let mut app = AppHandler::new(cpu, video, mem, cia1);
    app.run();
}
