use std::{
    fs::File, io::{Read, stdin}, path::PathBuf,
};

use clap::{Parser, Subcommand};

use crate::new_c64::{cia1::Cia1, cpu::model, memory::Memory, video::Video};

mod heap;
mod new_c64;

#[derive(Parser)]
#[command(name = "emulator", about = "An emulator for the 6502 and the c64")]
struct CmdArgs {
    #[command(subcommand)]
    subcommands: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
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
        #[arg(short, long, default_value = "test-resources/kernal.901227-03.bin")]
        kernal: PathBuf,
        #[arg(short, long, default_value = "test-resources/basic.901226-01.bin")]
        basic: PathBuf,
    },
    Heap,
}

fn main() {
    tracing_subscriber::fmt::init();
    let cmd_args = CmdArgs::parse();
    match cmd_args.subcommands {
        Subcommands::C64 { kernal, basic, verbose } => {
            new_c64::c64(&kernal, &basic, verbose);
        }
        Subcommands::Asm { file, origin } => {
            match file {
                Some(p) => asm(p.to_str(), origin),
                _ => asm(None, origin),
            };
        }
        Subcommands::Disasm { file, origin, end, kernal, basic } => disasm(file.to_str().unwrap(), origin, end, kernal, basic),
        Subcommands::Heap => run_heap(),
    }
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

fn disasm(file: &str, origin: Option<usize>, end: Option<usize>, kernal: PathBuf, basic: PathBuf) {
    let mut mem = Memory::new(&kernal, &basic);
    let mut mem_file = File::open(file).expect("Unable to open file {file}");
    let mut buf = Vec::new();
    mem_file.read_to_end(&mut buf).expect("Unable to read file {file}");
    let org = origin.unwrap_or(0);
    let len = buf.len();
    let end = end.unwrap_or(org + len - 3);
    mem.set_range(org..org+len, buf);
    let mut pc = org;
    let video = Video::default();
    let cia1 = Cia1::new();
    while pc < end {
        let (s, size) = model::disasm_and_len(pc as u16, &mut mem, &video, &cia1);
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
