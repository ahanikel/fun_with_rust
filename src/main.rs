use std::{cell::RefCell, io::Write, rc::Rc};

use tracing::info;

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

fn main() {
    c64();
}

#[allow(unused)]
fn wozmon() {
    tracing_subscriber::fmt::init();
    info!("Application starting");
    //let log1 = Rc::new(RefCell::new(String::new()));
    //let log = log1.clone();
    let mut log_fn = |s: &str| {
        //log.borrow_mut().push_str(s);
        //log.borrow_mut().push('\n');
        info!(s);
        //let _ = std::io::stdin().read(&mut [0;1]);
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
    let mut acia = Rc::new(RefCell::new(Acia::new(Some(Rc::new(RefCell::new(out_fn))))));
    let acia_sender = acia.borrow_mut().start();
    let mut mem = Memory::new();
    mem.register_device(wozmon, 0x8000, 0xffff);
    mem.register_device(acia, 0x5000, 0x5003);
    cpu.reset(&mut mem);
    loop {
        cpu.step(&mut mem);
    }
    let _ = acia_sender.send(Message::Quit);
}

#[allow(unused)]
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
