use std::{cell::RefCell, io::Write, rc::Rc};

use crate::{
    cpu6502::{
        acia::Acia,
        cpu::CPU,
        memory::{Memory, MemoryDevice, MemoryFromFile},
    },
    video::{AppHandler, Video},
};

mod cpu6502;
mod heap;
mod video;

fn main() {
    _run_heap();
    let log1 = Rc::new(RefCell::new(String::new()));
    let log = log1.clone();
    let mut log_fn = |s: &str| {
        log.borrow_mut().push_str(s);
        log.borrow_mut().push('\n');
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
    cpu.log_instructions = Some(&mut log_fn);
    let image = "test-resources/test-image";
    let wozmon = Box::new(MemoryFromFile::new(image));
    let mut acia = Box::new(Acia::new(Some(Rc::new(RefCell::new(out_fn)))));
    acia.start();
    let mut mem = Memory::new();
    mem.register_device(wozmon, 0x8000, 0xffff);
    mem.register_device(acia, 0x5000, 0x5003);
    let video_ram = Box::new(MemoryDevice::new(0x400));
    let color_ram = Box::new(MemoryDevice::new(0x400));
    mem.register_device(video_ram, 0x400, 0x7ff);
    mem.register_device(color_ram, 0xd800, 0xdbff);
    let video = Video::new(0x400, 0xd800);
    cpu.reset(&mut mem);
    let mut app = AppHandler::new(cpu, video, mem);
    app.run();
    // acia.stop()
}

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
