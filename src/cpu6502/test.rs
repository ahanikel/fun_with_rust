#![cfg(test)]
mod it;

use crate::cpu6502::model::opcode_from_instruction_and_mode;
use crate::cpu6502::*;

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
        cpu.step();
        println!("Step {step}: {}", cpu.status_line);
    }
    assert_eq!(0, cpu.cycle);
    assert_eq!(0xaa6c, cpu.pc);
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

fn _test_sbc(a: u8, b: u8, carry: bool, expected_res: u8, expected_overflow: bool, expected_carry: bool) {
    let mut cpu = CPU::new();
    let carry_inst = if carry { Instruction::SEC } else { Instruction::CLC };
    let prog = [
        opcode_from_instruction_and_mode(Instruction::LDA, AddrMode::Immediate),
        a,
        opcode_from_instruction_and_mode(carry_inst, AddrMode::Implied),
        opcode_from_instruction_and_mode(Instruction::SBC, AddrMode::Immediate),
        b,
    ];
    cpu.reset();
    cpu.pc = 0x1000;
    for (pos, b) in prog.iter().enumerate() {
        cpu.mem[0x1000 + pos] = *b;
    }
    for step in 0..14 {
        cpu.step();
        println!("Step {step}: {}", cpu.status_line);
    }
    assert_eq!(
        expected_res, cpu.a,
        "The unsigned result of {} - {} should be {}",
        a, b, expected_res
    );
    assert_eq!(
        expected_overflow,
        cpu.is_set(StatusFlag::Overflow),
        "The overflow flag should {}be set.",
        if expected_overflow { "" } else { "not " }
    );
    assert_eq!(
        expected_carry,
        cpu.is_set(StatusFlag::Carry),
        "The carry flag should {}be set.",
        if expected_carry { "" } else { "not " }
    );
}

#[test]
/**
 * Borrow occurs, but no overflow:
 * We compute 3 - 5.
 * Unsigned perspective: 3 - 5 = -2. . This is impossible in unsigned math, so it wraps to 254.
 *                                     Borrow occurs (carry = 0).
 * Signed perspective: +3 - (+5) = -2. In signed 8-bit math, 254 represents -2. The answer is
 *                                     perfectly correct and fits within the -128 to +127 range.
 *                                     No overflow (overflow = 0).
 */
fn test_sbc_1() {
    _test_sbc(3, 5, true, 0xfe, false, false);
    _test_sbc(3, 5, false, 0xfd, false, false);
}

#[test]
/**
 * Overflow occurs, but no borrow.
 * We compute -120 - 10, in hex: $88 (-120) minus $0A (10).
 * Unsigned perspective: 136 - 10 = 126 ($7E).
 *   Since 136 ≥ 10, no wrap below zero happened. No borrow (carry = 1).
 * Signed perspective: -120 - 10 = -130.
 *   However, -130 is too small to fit in a signed byte (limit is -128).
 *   The result wraps around to +126 ($7E). A negative minus a positive resulted in a positive.
 *   Overflow occurs (overflow = 1).
 */
fn test_sbc_2() {
    _test_sbc((-120_i8).cast_unsigned(), 10, true, 0x7e, true, true);
    _test_sbc((-120_i8).cast_unsigned(), 10, false, 0x7d, true, true);
}

#[test]
/**
 * Borrow occurs but no overflow.
 * We compute 250 - 252.
 * Unsigned perspective: 250 - 252 = -2.
 *   This is impossible in unsigned math, so it wraps to 254 ($FE).
 *   Borrow occurs (carry = 0).
 * Signed perspective: -6 - -4 = -2.
 *   The result fits well within the -128 to +127 range.
 *   No overflow (overflow = 0)
 */
fn test_sbc_3a() {
    _test_sbc(250, 252, true, 0xfe, false, false);
    _test_sbc(250, 252, false, 0xfd, false, false);
}

#[test]
/**
 * Similar to test_sbc_3a but we choose the values such that a borrow occurs with the
 * carry bit initially clear.
 */
fn test_sbc_3b() {
    _test_sbc(250, 250, false, 0xff, false, false);
}

#[test]
/**
 * Similar to test_sbc_3b but we choose the values such that no borrow occurs with the
 * carry bit initially clear.
 */
fn test_sbc_3c() {
    _test_sbc(251, 250, false, 0, false, true);
}

#[test]
fn test_sbc_3d() {
    _test_sbc(251, 255, false, 251, false, false);
}

#[test]
/**
 * Both borrow and overflow occur.
 * We compute 127 - -127
 * Unsigned perspective: 127 - 129 = -2.
 *   This is impossible in unsigned math, so it wraps to 254 ($FE).
 *   Borrow occurs (carry = 0).
 * Signed perspective: 127 - -127 = 254.
 *   The result does not fit within the -128 to +127 range.
 *   Overflow occurs (overflow = 1)
 */
fn test_sbc_4() {
    _test_sbc(127, (-127_i8).cast_unsigned(), true, 0xfe, true, false);
    _test_sbc(127, (-127_i8).cast_unsigned(), false, 0xfd, true, false);
}

#[test]
/**
 * Neither borrow nor overflow occur.
 * We compute -3 - 5.
 */
fn test_sbc_5() {
    _test_sbc(253, 5, true, 0xf8, false, true);
    _test_sbc(253, 5, false, 0xf7, false, true);
}

#[test]
/**
 * Neither borrow nor overflow occur.
 * We compute 126 - 123.
 */
fn test_sbc_6() {
    _test_sbc(126, 123, true, 3, false, true);
    _test_sbc(126, 123, false, 2, false, true);
}

#[test]
/**
 * Neither borrow nor overflow occur.
 * We compute 6 - 3.
 */
fn test_sbc_7() {
    _test_sbc(6,3, true, 3, false, true);
    _test_sbc(6,3, false, 2, false, true);
}

fn _test_adc(a: u8, b: u8, expected_res: u8, expected_overflow: bool, expected_carry: bool) {
    let mut cpu = CPU::new();
    let prog = [
        opcode_from_instruction_and_mode(Instruction::LDA, AddrMode::Immediate),
        a,
        opcode_from_instruction_and_mode(Instruction::CLC, AddrMode::Implied),
        opcode_from_instruction_and_mode(Instruction::ADC, AddrMode::Immediate),
        b,
    ];
    cpu.reset();
    cpu.pc = 0x1000;
    for (pos, b) in prog.iter().enumerate() {
        cpu.mem[0x1000 + pos] = *b;
    }
    for step in 0..14 {
        cpu.step();
        println!("Step {step}: {}", cpu.status_line);
    }
    assert_eq!(
        expected_res, cpu.a,
        "The unsigned result of {} + {} should be {}",
        a, b, expected_res
    );
    assert_eq!(
        expected_overflow,
        cpu.is_set(StatusFlag::Overflow),
        "The overflow flag should {}be set.",
        if expected_overflow { "" } else { "not " }
    );
    assert_eq!(
        expected_carry,
        cpu.is_set(StatusFlag::Carry),
        "The carry flag should {}be set.",
        if expected_carry { "" } else { "not " }
    );
}

#[test]
/**
 * Neither carry nor overflow occur.
 * We compute 3 + -5.
 * Unsigned perspective: 3 + 251 = 254.
 *   No borrow occurs (carry = 0).
 * Signed perspective: +3 + (-5) = -2.
 *   In signed 8-bit math, 254 represents -2. The answer is
 *   perfectly correct and fits within the -128 to +127 range.
 *   No overflow (overflow = 0).
 */
fn test_adc_1() {
    _test_adc(3, (-5_i8).cast_unsigned(), 0xfe, false, false);
}

#[test]
/**
 * Neither carry nor overflow occur.
 * We compute -120 + 10, in hex: $88 (-120) plus $0A (10).
 * Unsigned perspective: 136 + 10 = 146 ($92).
 *   Since 136 ≥ 10, no wrap below zero happened. No borrow (carry = 0).
 * Signed perspective: -120 + 10 = -110.
 *   The result is within range, so no overflow (overflow = 0).
 */
fn test_adc_2() {
    _test_adc((-120_i8).cast_unsigned(), 10, 0x92, false, false);
}

#[test]
/**
 * Carry but no overflow occur.
 * We compute 250 + 252.
 * Unsigned perspective: 250 + 252 = 502.
 *   This is impossible in unsigned math, so it wraps to 246 ($F6).
 *   Borrow occurs (carry = 1).
 * Signed perspective: -6 + -4 = -10.
 *   The result fits well within the -128 to +127 range.
 *   No overflow (overflow = 0)
 */
fn test_adc_3() {
    _test_adc(250, 252, 0xf6, false, true);
}

#[test]
/**
 * Carry but no overflow occur.
 * We compute 127 + -127
 * Unsigned perspective: 127 + 129 = 256.
 *   This is impossible in unsigned math, so it wraps to 0.
 *   Carry is set (carry = 1).
 * Signed perspective: 127 + -127 = 0.
 *   The result fits well within the -128 to +127 range.
 *   No overflow (overflow = 0)
 */
fn test_adc_4() {
    _test_adc(127, (-127_i8).cast_unsigned(), 0, false, true);
}

#[test]
/**
 * Carry but no overflow occur.
 * We compute -3 + 5.
 * Unsigned perspective: 253 + 5 = 258.
 *   This is impossible in unsigned math, so it wraps to 2.
 *   Carry is set (carry = 1).
 * Signed perspective: -3 + 5 = 2.
 *   The result fits well within the -128 to +127 range.
 *   No overflow (overflow = 0)
  */
fn test_adc_5() {
    _test_adc((-3_i8).cast_unsigned(), 5, 2, false, true);
}

#[test]
/**
 * No carry but overflow occurs.
 * We compute 126 + 123.
 * Unsigned perspective: 126 + 123 = 249.
 *   Carry is unset (carry = 0).
 * Signed perspective: 126 + 123 = 249.
 *   The result does not fit within the -128 to +127 range.
 *   Overflow occurs (overflow = 1)
  */
fn test_adc_6() {
    _test_adc(126, 123, 249, true, false);
}

#[test]
/**
 * Neither carry nor overflow occur.
 * We compute 6 + 3.
 */
fn test_adc_7() {
    _test_adc(6,3, 9, false, false);
}
