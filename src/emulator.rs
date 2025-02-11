use rand::prelude::*;
use std::error::Error;

#[derive(Debug)]
enum Src {
    Reg(usize),
    Literal(u16),
    Key,
    IReg,
}

#[derive(Debug)]
enum Ops {
    DisplayClear,
    DisplayUpdate(Src, Src, Src),
    CallSubRoutine(Src),
    ReturnSubRoutine,
    Jump(Src),
    JumpRelative(Src),
    JumpEq(Src, Src),
    JumpNeq(Src, Src),
    Add(Src, Src, Src),
    Sub(Src, Src, Src),
    And(Src, Src, Src),
    Or(Src, Src, Src),
    Xor(Src, Src, Src),
    LShift(Src),
    RShift(Src),
    Rand(Src, Src),
    ReadDelay(Src),
    GetKey(Src),
    WriteDelay(Src),
    WriteSound(Src),
    GetSprite(Src),
    BCD(Src),
    RegDump(Src),
    RegLoad(Src),
}

pub struct EmulatorState {
    // Going to seperate state out, for use in file io
    pub ram: Vec<u8>, // and anticipating emulator will need extra stuff
    pub pc: u16,
    pub ireg: u16,
    pub stack: Vec<u16>,
    pub stack_len: usize,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub register_bank: Vec<u8>,
    pub display: Vec<bool>,
}

pub struct Emulator {
    state: EmulatorState,
    rng_state: ThreadRng,
    prev_keys: u16,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            state: EmulatorState {
                ram: vec![0; 4096],
                pc: 0,
                ireg: 0,
                stack: vec![0; 32],
                stack_len: 0,
                delay_timer: 0,
                sound_timer: 0,
                register_bank: vec![0; 16],
                display: vec![false; 32 * 64],
            },
            rng_state: rand::rng(),
            prev_keys: 0,
        }
    }

    pub fn step(&mut self, keys: u16) {
        let opcode = self.fetch();
        if opcode != 0 {
            let op = self.decode(opcode);
            self.execute(op, keys);
            self.prev_keys = keys;
        }
    }

    pub fn get_state(&self) -> &EmulatorState {
        &self.state
    }

    pub fn load_state(&self, _state: &EmulatorState) -> Result<(), Box<dyn Error>> {
        todo!();
    }

    pub fn get_prog(&self) -> Vec<u8> {
        todo!();
    }

    pub fn load_prog(&mut self, prog: &[u8]) -> Result<(), Box<dyn Error>> {
        for i in 0..prog.len() {
            self.state.ram[512 + i] = prog[i]; // Offset for legacy reasons
        }
        self.state.pc = 512;
        Ok(())
    }

    fn fetch(&mut self) -> u16 {
        let opcode: u16 = ((self.state.ram[self.state.pc as usize] as u16) << 8)
            + (self.state.ram[(self.state.pc as usize) + 1] as u16);
        if opcode != 0 {
            self.state.pc += 2;
        }

        opcode
    }

    fn decode(&self, opcode: u16) -> Ops {
        match (opcode & 0xF000) >> 12 {
            0x0 => match opcode & 0x0FFF {
                0x0E0 => Ops::DisplayClear,
                0x0EE => Ops::ReturnSubRoutine,
                _ => panic!("Opcode {:04X} not supported", opcode),
            },
            0x1 => Ops::Jump(Src::Literal(opcode & 0x0FFF)),
            0x2 => Ops::CallSubRoutine(Src::Literal(opcode & 0x0FFF)),
            0x3 => Ops::JumpEq(
                Src::Reg(((opcode & 0xF00) >> 8) as usize),
                Src::Literal(opcode & 0x00FF),
            ),
            0x4 => Ops::JumpNeq(
                Src::Reg(((opcode & 0xF00) >> 8) as usize),
                Src::Literal(opcode & 0x00FF),
            ),
            0x5 => Ops::JumpEq(
                Src::Reg(((opcode & 0xF00) >> 8) as usize),
                Src::Reg(((opcode & 0x00F0) >> 8) as usize),
            ),
            0x6 => Ops::Add(
                Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                Src::Literal(opcode & 0x00FF),
                Src::Literal(0),
            ),
            0x7 => Ops::Add(
                Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                Src::Literal(opcode & 0x00FF),
            ),
            0x8 => match opcode & 0x000F {
                0x0 => Ops::Add(
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x00F0) >> 4) as usize),
                    Src::Literal(0),
                ),
                0x1 => Ops::Or(
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x00F0) >> 4) as usize),
                ),
                0x2 => Ops::And(
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x00F0) >> 4) as usize),
                ),
                0x3 => Ops::Xor(
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x00F0) >> 4) as usize),
                ),
                0x4 => Ops::Add(
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x00F0) >> 4) as usize),
                ),
                0x5 => Ops::Sub(
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x00F0) >> 4) as usize),
                ),
                0x6 => Ops::RShift(Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                0x7 => Ops::Sub(
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                    Src::Reg(((opcode & 0x00F0) >> 4) as usize),
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                ),
                0xE => Ops::LShift(Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                _ => unreachable!(),
            },
            0x9 => Ops::JumpNeq(
                Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                Src::Reg(((opcode & 0x00F0) >> 4) as usize),
            ),
            0xA => Ops::Add(Src::IReg, Src::Literal(opcode & 0x0FFF), Src::Literal(0)),
            0xB => Ops::JumpRelative(Src::Literal(opcode & 0x0FFF)),
            0xC => Ops::Rand(
                Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                Src::Literal(opcode & 0x00FF),
            ),
            0xD => Ops::DisplayUpdate(
                Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                Src::Reg(((opcode & 0x00F0) >> 4) as usize),
                Src::Literal(opcode & 0x000F),
            ),
            0xE => match opcode & 0x00FF {
                0x9E => Ops::JumpEq(Src::Key, Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                0xA1 => Ops::JumpNeq(Src::Key, Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                _ => unreachable!(),
            },
            0xF => match opcode & 0x00FF {
                0x07 => Ops::ReadDelay(Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                0x0A => Ops::GetKey(Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                0x15 => Ops::WriteDelay(Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                0x18 => Ops::WriteSound(Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                0x1E => Ops::Add(
                    Src::IReg,
                    Src::IReg,
                    Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                ),
                0x29 => Ops::GetSprite(Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                0x33 => Ops::BCD(Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                0x55 => Ops::RegDump(Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                0x65 => Ops::RegLoad(Src::Reg(((opcode & 0x0F00) >> 8) as usize)),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn execute(&mut self, op: Ops, keys: u16) {
        match op {
            Ops::DisplayClear => {
                for i in 0..self.state.display.len() {
                    self.state.display[i] = false;
                }
            }
            Ops::DisplayUpdate(Src::Reg(vx), Src::Reg(vy), Src::Literal(n)) => {
                let px = (self.state.register_bank[vx] & 0x3F) as usize;
                let py = (self.state.register_bank[vy] & 0x1F) as usize;
                self.state.register_bank[15] = 0;

                for offset in 0..(n as usize) {
                    if py + offset > 31 {
                        break;
                    }
                    let byte = self.state.ram[(self.state.ireg as usize) + offset];
                    for bit in 0..8 {
                        if px + (bit as usize) > 63 {
                            break;
                        }
                        if byte & (0x80 >> bit) > 0 {
                            let idx = (py + offset) * 64 + px + (bit as usize);
                            if self.state.display[idx] {
                                self.state.display[idx] = false;
                                self.state.register_bank[15] = 1;
                            } else {
                                self.state.display[idx] = true;
                            }
                        }
                    }
                }
            }
            Ops::DisplayUpdate(_, _, _) => panic!("Unsupported Display Update: {:?}", op),
            Ops::CallSubRoutine(Src::Literal(n)) => {
                if self.state.stack_len < 16 {
                    self.state.stack[self.state.stack_len] = self.state.pc;
                    self.state.stack_len += 1;
                    self.state.pc = n;
                } else {
                    panic!("Stack Overflow");
                }
            }
            Ops::CallSubRoutine(_) => panic!("Unsuppoorted CallSubRoutine: {:?}", op),
            Ops::ReturnSubRoutine => {
                if self.state.stack_len == 0 {
                    panic!("Stack Underflow");
                } else {
                    self.state.pc = self.state.stack[self.state.stack_len - 1];
                    self.state.stack_len -= 1;
                }
            }
            Ops::Jump(Src::Literal(n)) => {
                self.state.pc = n;
            }
            Ops::Jump(_) => panic!("Unsupported Jump: {:?}", op),
            Ops::JumpRelative(Src::Literal(n)) => {
                self.state.pc = (self.state.register_bank[0] as u16) + n;
            }
            Ops::JumpRelative(_) => panic!("Unsupported JumpRelative: {:?}", op),
            Ops::JumpEq(Src::Reg(vx), Src::Literal(n)) => {
                if self.state.register_bank[vx] == (n as u8) {
                    self.state.pc += 2;
                }
            }
            Ops::JumpEq(Src::Reg(vx), Src::Reg(vy)) => {
                if self.state.register_bank[vx] == self.state.register_bank[vy] {
                    self.state.pc += 2;
                }
            }
            Ops::JumpEq(Src::Key, Src::Reg(vx)) => {
                if keys & (1 << (self.state.register_bank[vx] as u16)) > 0 {
                    self.state.pc += 2;
                }
            }
            Ops::JumpEq(_, _) => panic!("Unsupported JumpEq: {:?}", op),
            Ops::JumpNeq(Src::Reg(vx), Src::Literal(n)) => {
                if self.state.register_bank[vx] != (n as u8) {
                    self.state.pc += 2;
                }
            }
            Ops::JumpNeq(Src::Reg(vx), Src::Reg(vy)) => {
                if self.state.register_bank[vx] != self.state.register_bank[vy] {
                    self.state.pc += 2;
                }
            }
            Ops::JumpNeq(Src::Key, Src::Reg(vx)) => {
                if keys & (1 << (self.state.register_bank[vx] as u16)) == 0 {
                    self.state.pc += 2;
                }
            }
            Ops::JumpNeq(_, _) => panic!("Unsupported JumpNeq: {:?}", op),
            Ops::Add(Src::IReg, Src::Literal(a), Src::Literal(b)) => {
                self.state.ireg = a + b;
            }
            Ops::Add(Src::IReg, Src::IReg, Src::Reg(vx)) => {
                self.state.ireg = self.state.ireg + (self.state.register_bank[vx] as u16);
            }
            Ops::Add(Src::Reg(vx), Src::Literal(a), Src::Literal(b)) => {
                self.state.register_bank[vx] = ((a + b) % 256) as u8;
            }
            Ops::Add(Src::Reg(vx), Src::Reg(vy), Src::Literal(n)) => {
                self.state.register_bank[vx] =
                    ((self.state.register_bank[vy] as u16 + n) % 256) as u8;
            }
            Ops::Add(Src::Reg(vd), Src::Reg(vx), Src::Reg(vy)) => {
                let res =
                    (self.state.register_bank[vx] as u16) + (self.state.register_bank[vy] as u16);
                self.state.register_bank[15] = ((res & 0x0100) >> 8) as u8;
                self.state.register_bank[vd] = (res & 0x00FF) as u8;
            }
            Ops::Add(_, _, _) => panic!("Unsupported Add: {:?}", op),
            Ops::Sub(Src::Reg(vd), Src::Reg(vx), Src::Reg(vy)) => {
                if self.state.register_bank[vy] > self.state.register_bank[vx] {
                    self.state.register_bank[15] = 0;
                    self.state.register_bank[vd] =
                        255 - (self.state.register_bank[vy] - self.state.register_bank[vx] - 1);
                } else {
                    self.state.register_bank[15] = 1;
                    self.state.register_bank[vd] =
                        self.state.register_bank[vx] - self.state.register_bank[vy];
                }

                let res =
                    (self.state.register_bank[vx] as u16) + (self.state.register_bank[vy] as u16);
                self.state.register_bank[vd] = (res & 0x00FF) as u8;
            }
            Ops::Sub(_, _, _) => panic!("Unsupported Sub: {:?}", op),
            Ops::Or(Src::Reg(vd), Src::Reg(vx), Src::Reg(vy)) => {
                self.state.register_bank[vd] =
                    self.state.register_bank[vx] | self.state.register_bank[vy];
            }
            Ops::Or(_, _, _) => panic!("Unsupported BitOr: {:?}", op),
            Ops::And(Src::Reg(vd), Src::Reg(vx), Src::Reg(vy)) => {
                self.state.register_bank[vd] =
                    self.state.register_bank[vx] & self.state.register_bank[vy];
            }
            Ops::And(_, _, _) => panic!("Unsupported BitAnd: {:?}", op),
            Ops::Xor(Src::Reg(vd), Src::Reg(vx), Src::Reg(vy)) => {
                self.state.register_bank[vd] =
                    self.state.register_bank[vx] ^ self.state.register_bank[vy];
            }
            Ops::Xor(_, _, _) => panic!("Unsupported BitXor: {:?}", op),
            Ops::RShift(Src::Reg(vx)) => {
                if self.state.register_bank[vx] & 0x01 > 0 {
                    self.state.register_bank[15] = 1;
                }
                self.state.register_bank[vx] = (self.state.register_bank[vx] & 0xFE) >> 1;
            }
            Ops::RShift(_) => panic!("Unsupported LShift: {:?}", op),
            Ops::LShift(Src::Reg(vx)) => {
                if self.state.register_bank[vx] & 0x80 > 0 {
                    self.state.register_bank[15] = 1;
                }
                self.state.register_bank[vx] = (self.state.register_bank[vx] & 0x7F) << 1;
            }
            Ops::LShift(_) => panic!("Unsupported LShift: {:?}", op),
            Ops::Rand(Src::Reg(vx), Src::Literal(n)) => {
                self.state.register_bank[vx] = self.rng_state.random::<u8>() & (n as u8);
            }
            Ops::Rand(_, _) => panic!("Unsupported Rand: {:?}", op),
            Ops::ReadDelay(Src::Reg(vx)) => {
                self.state.register_bank[vx] = self.state.delay_timer;
            }
            Ops::ReadDelay(_) => panic!("Unsupported GetDelay: {:?}", op),
            Ops::GetKey(Src::Reg(vx)) => {
                if keys != self.prev_keys {
                    for i in 0..16 {
                        if (keys ^ self.prev_keys) & (1 << i) > 0 {
                            self.state.register_bank[vx] = i as u8;
                            break;
                        }
                    }
                } else {
                    self.state.pc -= 1;
                }
            }
            Ops::GetKey(_) => panic!("Unsupported GetKey: {:?}", op),
            Ops::WriteDelay(Src::Reg(vx)) => {
                self.state.delay_timer = self.state.register_bank[vx];
            }
            Ops::WriteDelay(_) => panic!("Unsupported WriteDelay: {:?}", op),
            Ops::WriteSound(Src::Reg(vx)) => {
                self.state.sound_timer = self.state.register_bank[vx];
            }
            Ops::WriteSound(_) => panic!("Unsupported WriteSound: {:?}", op),
            Ops::GetSprite(Src::Reg(vx)) => {
                self.state.ireg = 0x0050 + (0x0005 * (self.state.register_bank[vx] as u16));
            }
            Ops::GetSprite(_) => panic!("Upsupported GetSprite: {:?}", op),
            Ops::BCD(Src::Reg(vx)) => {
                let mut val = self.state.register_bank[vx];
                self.state.ram[(self.state.ireg + 2) as usize] = val % 10;
                val = val / 10;
                self.state.ram[(self.state.ireg + 1) as usize] = val % 10;
                val = val / 10;
                self.state.ram[self.state.ireg as usize] = val % 10;
            }
            Ops::BCD(_) => panic!("Upsupported BCD: {:?}", op),
            Ops::RegDump(Src::Reg(vx)) => {
                for i in 0..=vx {
                    self.state.ram[(self.state.ireg + i as u16) as usize] =
                        self.state.register_bank[i as usize];
                }
            }
            Ops::RegDump(_) => panic!("Unsupported RegDump: {:?}", op),
            Ops::RegLoad(Src::Reg(vx)) => {
                for i in 0..=vx {
                    self.state.register_bank[i as usize] =
                        self.state.ram[(self.state.ireg + i as u16) as usize];
                }
            }
            Ops::RegLoad(_) => panic!("Unsupported RegLoad: {:?}", op),
        }
    }
}
