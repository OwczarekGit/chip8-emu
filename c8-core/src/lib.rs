pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug, Clone, Copy)]
pub struct Emulator {
    ram: [u8; RAM_SIZE],
    reg: [u8; NUM_REGS],
    i_reg: u16,
    pc: u16,
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    keys: [bool; NUM_KEYS],
    sp: u16,
    stack: [u16; STACK_SIZE],
    dt: u8,
    st: u8,
}

impl Emulator {
    pub fn state_string(&self) -> String {
        format!(
            "[ V00: {:02X} V01: {:02X} V02: {:02X} V03: {:02X} ]\n[ V04: {:02X} V05: {:02X} V06: {:02X} V07: {:02X} ]\n[ V08: {:02X} V09: {:02X} V10: {:02X} V11: {:02X} ]\n[ V12: {:02X} V13: {:02X} V14: {:02X} V15: {:02X} ]",
            self.reg[0],
            self.reg[1],
            self.reg[2],
            self.reg[3],
            self.reg[4],
            self.reg[5],
            self.reg[6],
            self.reg[7],
            self.reg[8],
            self.reg[9],
            self.reg[10],
            self.reg[11],
            self.reg[12],
            self.reg[13],
            self.reg[14],
            self.reg[15],
        )
    }

    pub fn snapshot(&self) -> Emulator {
        *self
    }

    pub fn display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, index: usize, pressed: bool) {
        self.keys[index] = pressed;
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + rom.len();
        self.ram[start..end].copy_from_slice(rom);
    }
}

impl Emulator {
    pub fn new() -> Self {
        let mut emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            reg: [0; NUM_REGS],
            i_reg: 0,
            keys: [false; NUM_KEYS],
            sp: 0,
            stack: [0; STACK_SIZE],
            dt: 0,
            st: 0,
        };
        emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.keys = [false; NUM_KEYS];
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }
}

impl Emulator {
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}

impl Emulator {
    fn fetch(&mut self) -> u16 {
        let hi = self.ram[self.pc as usize] as u16;
        let lo = self.ram[self.pc as usize + 1] as u16;
        self.pc += 2;
        (hi << 8) | lo
    }

    pub fn timers_step(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP!
            }
            self.st -= 1;
        }
    }

    pub fn step(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    fn execute(&mut self, op: u16) {
        let d1 = (op & 0xF000) >> 12;
        let d2 = (op & 0x0F00) >> 8;
        let d3 = (op & 0x00F0) >> 4;
        let d4 = (op & 0x000F) >> 0;

        match (d1, d2, d3, d4) {
            (0, 0, 0, 0) => {}
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            (0, 0, 0xE, 0xE) => self.pc = self.pop(),
            (1, _, _, _) => self.pc = op & 0x0FFF,
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = op & 0x0FFF;
            }
            (3, x, _, _) => {
                if self.reg[x as usize] == (op & 0x00FF) as u8 {
                    self.pc += 2;
                }
            }
            (4, x, _, _) => {
                if self.reg[x as usize] != (op & 0x00FF) as u8 {
                    self.pc += 2;
                }
            }
            (5, x, y, 0) => {
                if self.reg[x as usize] == self.reg[y as usize] {
                    self.pc += 2;
                }
            }
            (6, x, _, _) => self.reg[x as usize] = (op & 0x00FF) as u8,
            (7, x, _, _) => {
                self.reg[x as usize] = self.reg[x as usize].wrapping_add((op & 0x00FF) as u8)
            }
            (8, x, y, 0) => self.reg[x as usize] = self.reg[y as usize],
            (8, x, y, 1) => self.reg[x as usize] |= self.reg[y as usize],
            (8, x, y, 2) => self.reg[x as usize] &= self.reg[y as usize],
            (8, x, y, 3) => self.reg[x as usize] ^= self.reg[y as usize],
            (8, x, y, 4) => {
                let (res, of) = self.reg[x as usize].overflowing_add(self.reg[y as usize]);
                self.reg[x as usize] = res;
                self.reg[0xF] = if of { 1 } else { 0 };
            }
            (8, x, y, 5) => {
                let (res, of) = self.reg[x as usize].overflowing_sub(self.reg[y as usize]);
                self.reg[x as usize] = res;
                self.reg[0xF] = if of { 0 } else { 1 };
            }
            (8, x, _, 6) => {
                let drp = self.reg[x as usize] & 1;
                self.reg[x as usize] >>= 1;
                self.reg[0xF] = drp;
            }
            (8, x, y, 7) => {
                let (res, of) = self.reg[y as usize].overflowing_sub(self.reg[x as usize]);
                self.reg[x as usize] = res;
                self.reg[0xF] = if of { 0 } else { 1 };
            }
            (8, x, _, 0xE) => {
                let drp = (self.reg[x as usize] >> 7) & 1;
                self.reg[x as usize] <<= 1;
                self.reg[0xF] = drp;
            }
            (9, x, y, 0) => {
                if self.reg[x as usize] != self.reg[y as usize] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => self.i_reg = op & 0x0FFF,
            (0xB, _, _, _) => self.pc = (self.reg[0] as u16) + (op & 0x0FFF),
            (0xC, x, _, _) => self.reg[x as usize] = random() & ((op & 0x00FF) as u8),
            (0xD, _, _, _) => {
                let x = self.reg[d2 as usize] as u16;
                let y = self.reg[d3 as usize] as u16;
                let rows = d4;
                let mut flip = false;

                for y_line in 0..rows {
                    let addr = self.i_reg + y_line;
                    let pix = self.ram[addr as usize];

                    for x_line in 0..8 {
                        if pix & (0b1000_0000 >> x_line) != 0 {
                            let xc = ((x + x_line) as usize) % SCREEN_WIDTH;
                            let yc = ((y + y_line) as usize) % SCREEN_HEIGHT;

                            let index = xc + SCREEN_WIDTH * yc;
                            flip |= self.screen[index];
                            self.screen[index] ^= true;
                        }
                    }
                }

                if flip {
                    self.reg[0xF] = 1;
                } else {
                    self.reg[0xF] = 0;
                }
            }
            (0xE, x, 9, 0xE) => {
                if self.keys[self.reg[x as usize] as usize] {
                    self.pc += 2;
                }
            }
            (0xE, x, 0xA, 1) => {
                if !self.keys[self.reg[x as usize] as usize] {
                    self.pc += 2;
                }
            }
            (0xF, x, 0, 7) => self.reg[x as usize] = self.dt,
            (0xF, x, 0, 0xA) => {
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.reg[x as usize] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }
            (0xF, x, 1, 5) => self.dt = self.reg[x as usize],
            (0xF, x, 1, 8) => self.st = self.reg[x as usize],
            (0xF, x, 1, 0xE) => self.i_reg = self.i_reg.wrapping_add(self.reg[x as usize] as u16),
            (0xF, x, 2, 9) => self.i_reg = self.reg[x as usize] as u16 * 5,
            (0xF, x, 3, 3) => {
                let vx = self.reg[x as usize] as f32;

                let huns = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = huns;
                self.ram[self.i_reg as usize + 1] = tens;
                self.ram[self.i_reg as usize + 2] = ones;
            }
            (0xF, x, 5, 5) => {
                let iv = self.i_reg as usize;
                for i in 0..=(x as usize) {
                    self.ram[i + iv] = self.reg[i];
                }
            }
            (0xF, x, 6, 5) => {
                let iv = self.i_reg as usize;
                for i in 0..=(x as usize) {
                    self.reg[i] = self.ram[i + iv];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented op code: {op:04X}"),
        }
    }
}

fn random() -> u8 {
    rand::random()
}
