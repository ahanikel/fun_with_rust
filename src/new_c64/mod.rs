#![allow(unused)]

mod char_rom;
mod memory;
mod cpu;

use std::{io::Read, path::PathBuf};

pub fn c64(kernal: PathBuf, basic: PathBuf, verbose: bool) {

}

struct Oscillator {
    cpu: cpu::Cpu,
    video: Video,
    memory: memory::Memory,
}

impl Oscillator {
    fn start() {}
    fn stop() {}
    fn trigger() {}
}

struct Video;

impl Video {
    fn reset() {}
    fn trigger() {}
}

#[derive(Default, Debug)]
struct Cia1 {
    timer_a_val: u16,
    timer_a_cnt: u16,
    timer_a_int: bool,
    timer_b_val: u16,
    timer_b_cnt: u16,
    timer_b_int: bool,
}

impl Cia1 {
    fn new() -> Self {
        Self::default()
    }
    fn reset() {}
    fn set_timer_a(value: u16) {

    }
}

struct Cia2;

struct Sid;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_cia1_default() {
        let cia1 = Cia1::new();
        dbg!(cia1);
    }
}