mod cpu6502;
mod heap;
use cpu6502::CPU;
use cpu6502::Instruction;
use cpu6502::HasDescription;
use cpu6502::IsOriginal;
use cpu6502::AddrMode;

fn main() {
    _run_heap();
    let mut cpu: CPU = CPU::new();
    cpu.reset();
    cpu.step();
    println!("{} (Absolute {} original)", Instruction::LDA.desc(),
    if AddrMode::Absolute.is_original() {
        "is"
    } else {
        "is not"
    });
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
