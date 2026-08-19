use crate::cpu6502::*;
use crate::cpu6502::acia::*;
use std::cell::RefCell;
#[allow(unused)]
use std::io::Write;
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
    let s: String = String::from("\rFFF0.FFFF\r");
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
fn run_cpu_with_timer_interrupt(cpu: &mut CPU, no_cycles: u32) {
    for cycle in 1..=no_cycles {
        // every 20ms or 50 times per second
        if cycle % 20000 == 0 {
            cpu.irq = true;
        }
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
    let log1 = Rc::new(RefCell::new(String::new()));
    let log = log1.clone();
    let mut log_fn = |s: &str| {
        log.borrow_mut().push_str(s);
        log.borrow_mut().push('\n');
    };
    let out1 = Rc::new(RefCell::new(String::new()));
    let out = out1.clone();
    let out_fn = move |s: &str| {
        let mut o = out1.borrow_mut();
        for c in s.chars() {
            if c == '\r' {
                o.push('\n'); // Wozmon uses \r for line breaks
            } else {
                o.push(c);
            }
        }
    };
    let mut cpu = setup_cpu(Some(Rc::new(RefCell::new(out_fn))), Some(&mut log_fn));
    run_cpu(&mut cpu, 1000000);
    let out = out.take();
    let log = log.take();
    {
        let mut log_file = std::fs::File::create("/tmp/asm.log").unwrap();
        log_file.write(log.as_bytes()).unwrap();
    }
    assert_eq!([b'F', b'F', b'F', b'0', b'.', b'F', b'F', b'F', b'F', 0x0D, 0], &cpu.mem[0x200..=0x20a]);
    assert_eq!("\\\n\n\n\nFFFFFF00..FFFFFFFF\n\n\nFFF0: 00 00 00 00 00 00 00 00\nFFF8: 00 00 00 0F 1C 80 00 00\n", &out);
}