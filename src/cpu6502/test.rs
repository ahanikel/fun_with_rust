#[test]
fn test_brk_rti() {
    let mut cpu = super::CPU::new();
    cpu.reset();
    cpu.mem[0xfffe] = 0xee;
    cpu.mem[0xffff] = 0xee;
    cpu.mem[0xeef0] = 0x40; // RTI
    for step in 0..36 {
        print!("Step {step}: ");
        cpu.step();
    }
}

#[test]
fn test_jsr_ret() {
    let mut cpu = super::CPU::new();
    cpu.mem[0xfffc] = 0xcc;
    cpu.mem[0xfffd] = 0xcc;
    cpu.mem[0xcccc] = 0x20; // JSR
    cpu.mem[0xcccd] = 0xdd;
    cpu.mem[0xccce] = 0xdd;
    cpu.mem[0xdddd] = 0x60; // RTS
    cpu.reset();
    for step in 0..36 {
        print!("Step {step}: ");
        cpu.step();
    }
}

#[test]
fn test_beq() {
    let mut cpu = super::CPU::new();
    cpu.mem[0xfffc] = 0xaa;
    cpu.mem[0xfffd] = 0xaa;
    cpu.mem[0xaaaa] = 0xf0; // BEQ
    cpu.mem[0xaaab] = 0xc0;
    cpu.reset();
    cpu.set_flag(super::StatusFlag::Zero);
    for step in 0..36 {
        print!("Step {step}: ");
        cpu.step();
    }
}
