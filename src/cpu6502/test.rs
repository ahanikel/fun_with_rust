#[cfg(test)]

use super::*;

#[test]
fn test_brk_rti() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.mem[0xfffe] = 0xee;
    cpu.mem[0xffff] = 0xee;
    cpu.mem[0xeef0] = 0x40; // RTI
    for step in 0..14 {
        print!("Step {step}: ");
        cpu.step();
    }
    assert!(cpu.is_set(StatusFlag::BRK));
    assert!(cpu.is_set(StatusFlag::IRQDisable));
    assert_eq!(0xeeee, cpu.pc);
    for step in 14..16 {
        print!("Step {step}: ");
        cpu.step();
    }
    assert!(cpu.is_set(StatusFlag::BRK));
    assert!(cpu.is_set(StatusFlag::IRQDisable));
    assert_eq!(0xeef0, cpu.pc);
    for step in 16..18 {
        print!("Step {step}: ");
        cpu.step();
    }
    assert!(cpu.is_clear(StatusFlag::BRK));
    assert!(cpu.is_clear(StatusFlag::IRQDisable));
    assert_eq!(0x0002, cpu.pc);
}

#[test]
fn test_jsr_ret() {
    let mut cpu = CPU::new();
    cpu.mem[0xfffc] = 0xcc;
    cpu.mem[0xfffd] = 0xcc;
    cpu.mem[0xcccc] = 0x20; // JSR
    cpu.mem[0xcccd] = 0xdd;
    cpu.mem[0xccce] = 0xdd;
    cpu.mem[0xdddd] = 0x60; // RTS
    cpu.reset();
    for step in 0..13 {
        print!("Step {step}: ");
        cpu.step();
    }
    assert_eq!(0xdddd, cpu.pc);
    for step in 13..15 {
        print!("Step {step}: ");
        cpu.step();
    }
    assert_eq!(0xcccf, cpu.pc);
}

#[test]
fn test_beq_taken() {
    let mut cpu = CPU::new();
    cpu.mem[0xfffc] = 0xaa;
    cpu.mem[0xfffd] = 0xaa;
    cpu.mem[0xaaaa] = 0xf0; // BEQ
    cpu.mem[0xaaab] = 0xc0;
    cpu.reset();
    cpu.set_flag(StatusFlag::Zero);
    for step in 0..10 {
        print!("Step {step}: ");
        cpu.step();
    }
    assert_eq!(0, cpu.cycle);
    assert_eq!(0xaa6a, cpu.pc);
}

#[test]
fn test_beq_not_taken() {
    let mut cpu = CPU::new();
    cpu.mem[0xfffc] = 0xaa;
    cpu.mem[0xfffd] = 0xaa;
    cpu.mem[0xaaaa] = 0xf0; // BEQ
    cpu.mem[0xaaab] = 0xc0;
    cpu.reset();
    cpu.clear_flag(StatusFlag::Zero);
    for step in 0..9 {
        print!("Step {step}: ");
        cpu.step();
    }
    assert_eq!(0, cpu.cycle);
    assert_eq!(0xaaac, cpu.pc);
}

#[test]
fn test_cmp_zpx_ind_lt() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.pc = 0x1000;
    cpu.a = 0x55;
    cpu.x = 1;
    cpu.mem[0x50] = 0x99;
    cpu.mem[0x51] = 0x99;
    cpu.mem[0x1000] = 0xc1; // CMP (zp,x)
    cpu.mem[0x1001] = 0x4f;
    cpu.mem[0x9999] = 0xf0;
    for step in 0..13 {
        print!("Step {step}: ");
        cpu.step();
    }
    assert!(cpu.is_set(StatusFlag::Negative));
    assert!(cpu.is_clear(StatusFlag::Zero));
    assert!(cpu.is_clear(StatusFlag::Carry));
}

#[test]
fn test_cmp_zpx_ind_eq() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.pc = 0x1000;
    cpu.a = 0x55;
    cpu.x = 1;
    cpu.mem[0x50] = 0x99;
    cpu.mem[0x51] = 0x99;
    cpu.mem[0x1000] = 0xc1; // CMP (zp,x)
    cpu.mem[0x1001] = 0x4f;
    cpu.mem[0x9999] = 0x55;
    for step in 0..13 {
        print!("Step {step}: ");
        cpu.step();
    }
    assert!(cpu.is_clear(StatusFlag::Negative));
    assert!(cpu.is_set(StatusFlag::Zero));
    assert!(cpu.is_set(StatusFlag::Carry));
}

#[test]
fn test_cmp_zpx_ind_gt() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.pc = 0x1000;
    cpu.a = 0xfe;
    cpu.x = 1;
    cpu.mem[0x50] = 0x99;
    cpu.mem[0x51] = 0x99;
    cpu.mem[0x1000] = 0xc1; // CMP (zp,x)
    cpu.mem[0x1001] = 0x4f;
    cpu.mem[0x9999] = 0x55;
    for step in 0..13 {
        print!("Step {step}: ");
        cpu.step();
    }
    assert!(cpu.is_clear(StatusFlag::Negative));
    assert!(cpu.is_clear(StatusFlag::Zero));
    assert!(cpu.is_set(StatusFlag::Carry));
}