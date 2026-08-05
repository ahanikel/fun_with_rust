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
    _run_heap();
    let mut cpu: CPU = CPU::new();
    cpu.reset();
    cpu.step();
    println!(
        "{} (Absolute {} original)",
        Instruction::LDA.desc(),
        if AddrMode::Absolute.is_original() {
            "is"
        } else {
            "is not"
        }
    );
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
