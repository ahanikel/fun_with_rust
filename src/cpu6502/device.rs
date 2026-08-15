/**
 * A memory-mapped peripheral. The CPU routes loads and stores at the
 * registered address to `read`/`write` instead of to `mem`.
 */
pub trait Device {
    fn read(&mut self, reg: u8) -> u8;
    fn write(&mut self, reg: u8, byte: u8);
}
