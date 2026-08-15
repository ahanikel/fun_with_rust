use std::{cell::RefCell, env::args, io::Read, rc::Rc};

use crate::cpu6502::{
    cpu::CPU,
    device::Device,
};

mod cpu6502;
mod heap;

fn main() {
    //let image = args().nth(1).unwrap_or("/Users/axel/src/lispos8/build/lispos8.bin".to_string());
    let image = "/Users/axel/src/lispos8/build/lispos8.bin";
    _run_heap();
    let mut cpu: CPU = CPU::new();
    let mut f = std::fs::File::open(image).expect("Give a file of a memory image as the first argument");
    f.read_exact(&mut cpu.mem[32768..]).unwrap();
    let acia = cpu6502::acia::Acia::new(String::from("ffe0.ffff"));
    let acia: Rc<RefCell<dyn Device>> = Rc::new(RefCell::new(acia));
    cpu.register_device(0x5000, acia.clone());
    cpu.register_device(0x5001, acia.clone());
    cpu.register_device(0x5002, acia.clone());
    cpu.register_device(0x5003, acia);
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
        assert!(alloc % 4 == 0);
        allocs.push(alloc);
    }
    for alloc in allocs.iter().rev() {
        heap.free(*alloc);
    }
}
