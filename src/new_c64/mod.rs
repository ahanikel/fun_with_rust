mod char_rom;
mod memory;
mod cpu;
mod video;
mod cia1;
mod app_handler;

use std::path::Path;

use tracing::info;

use crate::new_c64::{app_handler::AppHandler, cia1::Cia1, cpu::Cpu, memory::Memory, video::Video};

pub fn c64(kernal: &Path, basic: &Path, verbose: bool) {
    info!("C64 starting");
    let mut cpu = Cpu::new(verbose);
    let mut video = Video::default();
    let mut mem = Memory::new(kernal, basic);
    let mut cia1 = Cia1::new();
    mem.write(&mut video, &mut cia1, 1, 0b1110_0000); // enable ROMs
    cpu.reset(&mut mem, &mut video, &mut cia1);
    let mut app = AppHandler::new(cpu, video, mem, cia1);
    app.run();
}
