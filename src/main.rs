use std::{cell::RefCell, io::Write, path::PathBuf, rc::Rc};

use tracing::info;
use clap::Parser;

use crate::{
    cpu6502::{
        acia::{Acia, Message},
        cpu::CPU,
        memory::{Memory, MemoryDevice, MemoryFromFile},
    }, video::{AppHandler, Video, cia1::Cia1, control::Control},
};

mod cpu6502;
mod heap;
mod video;

#[derive(Parser)]
#[command(name = "emulator", about = "An emulator for the 6502 and the c64")]
struct CmdArgs {
    #[arg(long, default_value = "true")]
    wozmon: bool,
    #[arg(long, default_value = "false")]
    c64: bool,
    #[arg(long, default_value = "test-resources/kernal.901227-03.bin")]
    kernal: PathBuf,
    #[arg(long, default_value = "test-resources/basic.901226-01.bin")]
    basic: PathBuf,
}

fn main() {
    let cmd_args = CmdArgs::parse();
    if cmd_args.c64 {
        c64();
    } else {
        wozmon();
    }
}

fn wozmon() {
    tracing_subscriber::fmt::init();
    info!("Application starting");
    #[allow(unused)]
    let mut log_fn = |s: &str| {
        info!(s);
    };
    let out_fn = |b: u8| {
        if b == b'\r' {
            std::io::stdout().write_all(b"\n").unwrap(); // Wozmon uses \r for line breaks
        } else {
            std::io::stdout().write_all(&[b]).unwrap();
        }
        std::io::stdout().flush().unwrap();
    };
    let mut cpu: CPU = CPU::new();
    //cpu.log_instructions = Some(&mut log_fn);
    cpu.log_instructions = None;
    let wozmon = Rc::new(RefCell::new(MemoryFromFile::new( "test-resources/test-image")));
    let acia = Rc::new(RefCell::new(Acia::new(Some(Rc::new(RefCell::new(out_fn))))));
    #[allow(unused)]
    let acia_sender = acia.borrow_mut().start();
    let mut mem = Memory::new();
    mem.register_device(wozmon, 0x8000, 0xffff);
    mem.register_device(acia.clone(), 0x5000, 0x5003);
    cpu.reset(&mut mem);
    loop {
        cpu.step(&mut mem);
        if let Some(thr) = &acia.borrow().input_thread {
            if thr.is_finished() {
                break;
            }
        }
    }
}

fn c64() {
    tracing_subscriber::fmt::init();
    info!("Application starting");
    let out_fn = |b: u8| {
        if b == b'\r' {
            std::io::stdout().write_all(b"\n").unwrap();
        } else {
            std::io::stdout().write_all(&[b]).unwrap();
        }
        std::io::stdout().flush().unwrap();
    };
    let mut log_fn = |s: &str| {
        info!(s);
    };
    let mut cpu: CPU = CPU::new();
    //cpu.log_instructions = None;
    cpu.log_instructions = Some(&mut log_fn);
    let mut mem = Memory::new();
    let video_ram = Rc::new(RefCell::new(MemoryDevice::new(0x400)));
    mem.register_device(video_ram, 0x400, 0x7ff);
    let color_ram = Rc::new(RefCell::new(MemoryDevice::new(0x400)));
    mem.register_device(color_ram, 0xd800, 0xdbff);
    let kernal = Rc::new(RefCell::new(MemoryFromFile::new("test-resources/kernal.901227-03.bin")));
    mem.register_device(kernal, 0xe000, 0xffff);
    let basic = Rc::new(RefCell::new(MemoryFromFile::new("test-resources/basic.901226-01.bin")));
    mem.register_device(basic, 0xa000, 0xbfff);
    let control = Rc::new(RefCell::new(Control::new()));
    let video = Video::new(control.clone(), 0x400, 0xd800);
    mem.register_device(control, 0xd000, 0xd3ff);
    cpu.reset(&mut mem);
    let cia1 = Cia1::new();
    let mut app = AppHandler::new(cpu, video, mem, cia1);
    info!("Running event loop");
    app.run();
}

#[allow(unused)]
fn _run_heap() {
    let mut heap = heap::Heap::new();
    let mut allocs = Vec::new();
    for _ in 0..16383 {
        let alloc = heap.malloc(1);
        assert!(alloc.is_ok());
        let alloc = alloc.unwrap();
        assert!(alloc.is_multiple_of(4));
        allocs.push(alloc);
    }
    for alloc in allocs.iter().rev() {
        heap.free(*alloc);
    }
}
