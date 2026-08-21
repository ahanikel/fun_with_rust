use std::{
    cell::RefCell,
    io::{Read, Write},
    rc::Rc,
};

use crate::cpu6502::{acia::Acia, cpu::CPU, device::Device};

mod cpu6502;
mod heap;

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
    let mut f = std::fs::File::open(image).unwrap();
    f.read_exact(&mut cpu.mem[32768..]).unwrap();
    let acia = Acia::new(Some(Rc::new(RefCell::new(out_fn))));
    let acia: Rc<RefCell<dyn Device>> = Rc::new(RefCell::new(acia));
    for addr in 0x5000..=0x5003 {
        cpu.register_device(addr, acia.clone());
    }
    cpu.reset();
    loop {
        cpu.step();
    }
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
