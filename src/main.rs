use std::{cell::RefCell, io::Read, rc::Rc, thread::sleep, time::Duration};

use crate::cpu6502::{
    cpu::CPU,
    device::Device,
};

mod cpu6502;
mod heap;

fn main() {
    let image = "test-resources/test-image";
    _run_heap();
    let mut cpu: CPU = CPU::new();
    let mut f = std::fs::File::open(image).expect("Give a file of a memory image as the first argument");
    f.read_exact(&mut cpu.mem[32768..]).unwrap();
    let acia = cpu6502::acia::Acia::new(None);
    let acia: Rc<RefCell<dyn Device>> = Rc::new(RefCell::new(acia));
    cpu.register_device(0x5000, acia.clone());
    cpu.register_device(0x5001, acia.clone());
    cpu.register_device(0x5002, acia.clone());
    cpu.register_device(0x5003, acia);
    cpu.reset();
    let mut now = std::time::Instant::now();
    let mut elapsed = Duration::default();
    let mut last_irq = std::time::Instant::now();
    loop {
        cpu.irq = false;
        if cpu.cycle == 0 && (std::time::Instant::now() - last_irq > Duration::from_millis(50)) {
            cpu.irq = true;
            last_irq = std::time::Instant::now(); 
        }
        cpu.step();
        elapsed += std::time::Instant::now() - now;
        if cpu.cycle < cpu.cycles && cpu.cycle == cpu.cycles - 1 {
            let expected = Duration::from_micros(cpu.cycles.into());
            if elapsed < expected {
                let wait_for = expected - elapsed;
                //eprintln!("Sleeping for {}µs", wait_for.as_micros());
                sleep(wait_for);
            } else {
                //eprintln!("Took {}µs for {} cycles", elapsed.as_micros(), cpu.cycles);
            }
            now = std::time::Instant::now();
            elapsed = Duration::default();
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
