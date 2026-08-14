use std::{env::args, io::Read};

use crate::cpu6502::{
    cpu::CPU,
    model::{
        IsOriginal,
        addr_mode::AddrMode,
        instruction::{HasDescription, Instruction},
    },
};

mod cpu6502;
mod heap;

fn main() {
    let image = args().next().unwrap_or("/Users/axel/src/lispos8/build/lispos8.bin".to_string());
    _run_heap();
    let mut cpu: CPU = CPU::new();
    let mut f = std::fs::File::open(image).expect("Give a file of a memory image as the first argument");
    f.read_exact(&mut cpu.mem[32768..]).unwrap();
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
