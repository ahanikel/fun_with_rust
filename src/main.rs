use std::{
    cell::RefCell,
    io::{Write, stdin},
    path::PathBuf,
    rc::Rc,
};

use clap::{Parser, Subcommand};
use tracing::info;

use crate::{
    cpu6502::{
        acia::Acia,
        cpu::CPU,
        memory::{Memory, MemoryDevice, MemoryFromFile},
        model,
    },
    video::{AppHandler, Video, cia1::Cia1, control::Control},
};

mod cpu6502;
mod heap;
mod video;

#[derive(Parser)]
#[command(name = "emulator", about = "An emulator for the 6502 and the c64")]
struct CmdArgs {
    #[command(subcommand)]
    subcommands: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    Wozmon,
    C64 {
        #[arg(short, long, default_value = "test-resources/kernal.901227-03.bin")]
        kernal: PathBuf,
        #[arg(short, long, default_value = "test-resources/basic.901226-01.bin")]
        basic: PathBuf,
        #[arg(short, long)]
        verbose: bool,
    },
    Asm {
        #[arg(long, short)]
        file: Option<PathBuf>,
        #[arg(long, short)]
        origin: Option<usize>,
    },
    Disasm {
        file: PathBuf,
        #[arg(long, short)]
        origin: Option<usize>,
        end: Option<usize>,
    },
    Heap,
}

fn main() {
    let cmd_args = CmdArgs::parse();
    match cmd_args.subcommands {
        Subcommands::Wozmon => wozmon(),
        Subcommands::C64 { kernal, basic, verbose } => {
            c64(kernal.to_str().unwrap(), basic.to_str().unwrap(), verbose)
        }
        Subcommands::Asm { file, origin } => {
            match file {
                Some(p) => asm(p.to_str(), origin),
                _ => asm(None, origin),
            };
        }
        Subcommands::Disasm { file, origin, end } => disasm(file.to_str().unwrap(), origin, end),
        Subcommands::Heap => run_heap(),
    }
}

fn wozmon() {
    tracing_subscriber::fmt::init();
    info!("Application starting");
    let out_fn = |b: u8| {
        if b == b'\r' {
            std::io::stdout().write_all(b"\n").unwrap(); // Wozmon uses \r for line breaks
        } else {
            std::io::stdout().write_all(&[b]).unwrap();
        }
        std::io::stdout().flush().unwrap();
    };
    let mut cpu: CPU = CPU::new();
    cpu.log_instructions = None;
    let wozmon = Rc::new(RefCell::new(MemoryFromFile::new(
        "test-resources/test-image",
    )));
    let acia = Rc::new(RefCell::new(Acia::new(Some(Rc::new(RefCell::new(out_fn))))));
    #[allow(unused)]
    let acia_sender = acia.borrow_mut().start();
    let mut mem = Memory::new();
    mem.register_device(wozmon, 0x8000, 0xffff);
    mem.register_device(acia.clone(), 0x5000, 0x5003);
    cpu.reset(&mut mem);
    loop {
        cpu.step(&mut mem);
        if let Some(thr) = &acia.borrow().input_thread {
            if thr.is_finished() {
                break;
            }
        }
    }
}

fn c64(kernal_file: &str, basic_file: &str, verbose: bool) {
    tracing_subscriber::fmt::init();
    info!("Application starting");
    let mut log_fn = |s: &str| {
        info!(s);
    };
    let mut cpu: CPU = CPU::new();
    if verbose {
        cpu.log_instructions = Some(&mut log_fn);
    }
    let mut mem = Memory::new();
    let video_ram = Rc::new(RefCell::new(MemoryDevice::new(0x400)));
    mem.register_device(video_ram, 0x400, 0x7ff);
    let color_ram = Rc::new(RefCell::new(MemoryDevice::new(0x400)));
    mem.register_device(color_ram, 0xd800, 0xdbff);
    let kernal = Rc::new(RefCell::new(MemoryFromFile::new(kernal_file)));
    mem.register_device(kernal, 0xe000, 0xffff);
    let basic = Rc::new(RefCell::new(MemoryFromFile::new(basic_file)));
    mem.register_device(basic, 0xa000, 0xbfff);
    let control = Rc::new(RefCell::new(Control::new()));
    let video = Video::new(control.clone(), 0x400, 0xd800);
    mem.register_device(control, 0xd000, 0xd3ff);
    cpu.reset(&mut mem);
    let cia1 = Cia1::new();
    let mut app = AppHandler::new(cpu, video, mem, cia1);
    info!("Running event loop");
    app.run();
}

fn run_heap() {
    let mut heap = heap::Heap::new();
    let mut allocs = Vec::new();
    for _ in 0..16383 {
        let alloc = heap.malloc(1);
        assert!(alloc.is_ok());
        let alloc = alloc.unwrap();
        assert!(alloc.is_multiple_of(4));
        allocs.push(alloc);
    }
    for alloc in allocs.iter().rev() {
        heap.free(*alloc);
    }
}

fn disasm(file: &str, origin: Option<usize>, end: Option<usize>) {
    let mut mem = Memory::new();
    let mem_file = Rc::new(RefCell::new(MemoryFromFile::new(file)));
    let org = origin.unwrap_or(0);
    let len = mem_file.borrow().mem.len();
    let end = end.unwrap_or(org + len - 3);
    mem.register_device(mem_file, org, org + len);
    let mut pc = org;
    while pc < end {
        let (s, size) = model::disasm_and_len(pc as u16, &mut mem);
        println!("{:04X} {}", pc, s);
        pc += size as usize;
    }
}

fn asm(file: Option<&str>, origin: Option<usize>) {
    let mut file: Box<dyn std::io::BufRead> = match file {
        Some(f) => Box::new(std::io::BufReader::new(std::fs::File::open(f).unwrap())),
        None => Box::new(std::io::BufReader::new(stdin())),
    };
    let origin = origin.unwrap_or(0);
    let mut buf = String::new();
    let mut pc = origin;
    while let Ok(n) = file.read_line(&mut buf) && n > 3 {
        let bytes = model::asm(buf.as_str().trim_end(), origin as u16);
        match bytes {
            Ok(bytes) => {
                print!("{:04X} ", pc);
                for num in &bytes {
                    print!(" {:02X}", num);
                }
                println!();
                pc += bytes.len();
            }
            Err(e) => {
                eprintln!("asm failed: {}", e);
                eprintln!("Note that asm is very strict in what it accepts: exactly one space between opcode and argument (if any),");
                eprintln!("no leading or trailing spaces or newlines.")
            }
        }
        buf.clear();
    }
}
