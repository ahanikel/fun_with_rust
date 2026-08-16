use crate::cpu6502::*;
use crate::cpu6502::acia::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

#[allow(unused)]
fn setup_cpu<'a>(log_output: Option<Rc<RefCell<dyn FnMut(&str)>>>, log_instructions: Option<&'a mut dyn FnMut(&str)>) -> CPU<'a> {
    use std::{cell::RefCell, io::Read, rc::Rc};
    use crate::cpu6502::device::Device;

    let image = "test-resources/test-image";
    let mut cpu: CPU<'a> = CPU::new();
    cpu.log_instructions = log_instructions;
    let mut f = std::fs::File::open(image).unwrap();
    f.read_exact(&mut cpu.mem[32768..]).unwrap();
    let s: String = String::from("fff0.ffff\r\n");
    let mut acia = Acia::new(log_output);
    acia.set_input(&s);
    let acia: Rc<RefCell<dyn Device>> = Rc::new(RefCell::new(acia));
    for addr in 0x5000..=0x5003 {
        cpu.register_device(addr, acia.clone());
    }
    cpu.reset();
    cpu
}

#[allow(unused)]
fn run_cpu(cpu: &mut CPU, no_cycles: u32) {
    for cycle in 0..no_cycles {
        cpu.step();
    }
}

#[allow(unused)]
fn run_cpu_with_timing(cpu: &mut CPU, no_cycles: u32) {
    let mut now = std::time::Instant::now();
    let mut elapsed = Duration::default();
    let mut last_irq = std::time::Instant::now();
    loop {
        cpu.irq = false;
        cpu.irq_prev = false;
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
                std::thread::sleep(wait_for);
            }
            now = std::time::Instant::now();
            elapsed = Duration::default();
        }
    }
}

#[test]
fn test_input() {
    let mut log: String = String::new();
    let mut log_fn = |s: &str| {
        log.push_str(s);
        log.push('\n');
    };
    let out1 = Rc::new(RefCell::new(String::new()));
    let out = out1.clone();
    let out_fn = move |s: &str| {
        out1.borrow_mut().push_str(s);
    };
    let mut cpu = setup_cpu(Some(Rc::new(RefCell::new(out_fn))), Some(&mut log_fn));
    run_cpu(&mut cpu, 50000);
    assert_eq!("\r\nStarting Wozmon...\r\n\\\r\n".to_owned(), out.take());
    println!("{:?}", &cpu.mem[0x300..0x320]);
}