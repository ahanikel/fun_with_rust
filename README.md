# Rust Sandbox
Not sure what will come out of this but I'm
currently having fun learning Rust with these
components:

1. An simple implementation of a heap memory allocator.
2. A virtual 6502 CPU plus some devices like a UART and perhaps the video chip.

Perhaps I'll try implementing a simple Lisp if that's at all possible with 64K.

# Running a Eater-like machine with wozmon
```
cargo build
./target/debug/emulator wozmon
```

# Running the C64 emulator
```
cargo build
./target/debug/emulator c64
```

# Debugging tests
```
cargo test --no-run
rust-lldb target/debug/deps/emulator-ea3952e6dd030e75
(lldb) b emulator::new_c64::cia1::test::test_1
(lldb) r new_c64::cia1::test::test_1 --exact --no-capture
(lldb) p self.cpu.tmp_addr
(unsigned short) 56320
(lldb) dwim-print  -fX -- self.cpu.tmp_addr
(unsigned short) 0xDC00
```
