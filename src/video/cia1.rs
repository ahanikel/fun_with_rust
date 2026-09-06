use crate::cpu6502::memory::Device;

pub struct Cia1 {
    pub port_a: u8, // write to select which keyboard row to read
    pub port_b: u8, // read column from selected row
    pub ddr_a: u8,
    pub ddr_b: u8,
    // Keyboard matrix (8x8):
    // each byte represents a row, each bit represents a column, 
    // 0 = pressed, 1 = released
    pub matrix: [u8; 8],
}

impl Cia1 {
    pub fn new() -> Self {
        Self {
            port_a: 0xff,
            port_b: 0xff,
            ddr_a: 0,
            ddr_b: 0,
            matrix: [0xff; 8],
        }
    }
    pub fn set_key_state(&mut self, row: usize, col: usize, is_pressed: bool) {
        if is_pressed {
            self.matrix[row] &= !(1 << col);
        } else {
            self.matrix[row] |= 1 << col;
        }
    }
}

impl Device for Cia1 {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // port_a: output pins reflect port_a, input pins float "high"
            0 => (self.port_a & self.ddr_a) | !self.ddr_a,
            // port_b: matrix scan logic
            1 => {
                let mut res = 0xff;
                let row_sel = (self.port_a & self.ddr_a) | !self.ddr_a;
                for row in 0..8 {
                    // active low
                    if (row_sel & (1 << row)) == 0 {
                        res &= self.matrix[row];
                    }
                }
                (res & !self.ddr_b) | (self.port_b & self.ddr_b)
            }
            _ => panic!("Should not happen (misconfiguration)"),
        }
    }

    fn write(&mut self, addr: u16, byte: u8) {
        match addr {
            0 => self.port_a = byte,
            1 => self.port_b = byte,
            _ => panic!("Should not happen (misconfiguration)"),
        }
    }
}