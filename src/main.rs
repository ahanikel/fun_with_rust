use std::io::Read;

mod cpu6502;
mod heap;

fn main() {
    let mut cpu = cpu6502::CPU::new();
    cpu.reset();
    cpu.store_memory_word(0xfffe, 0xeeee);
    cpu.store_memory_byte(0xeef0, 0x40); // RTI
    let mut b: [u8; 1] = [0; 1];
    loop {
        let _ = std::io::stdin().read(&mut b).unwrap();
        if b[0] == b'q' {
            break;
        } else {
            cpu.step();
        }
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
