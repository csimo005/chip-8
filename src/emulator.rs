use std::error::Error;

#[derive(Debug)]
enum Src {
    Reg(usize),
    Literal(u16),
    //    Stack,
    IReg,
}

#[derive(Debug)]
enum Ops {
    DisplayClear,
    DisplayUpdate(Src, Src, Src),
    //    SubRoutine(Src),
    Jump(Src),
    //    JumpEq(Src, Src, Src),
    //    JumpNeq(Src, Src, Src),
    Add(Src, Src, Src),
}

pub struct EmulatorState {
    // Going to seperate state out, for use in file io
    pub ram: Vec<u8>, // and anticipating emulator will need extra stuff
    pub pc: u16,
    pub ireg: u16,
    pub stack: Vec<u16>,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub register_bank: Vec<u8>,
    pub display: Vec<bool>,
}

pub struct Emulator {
    state: EmulatorState,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            state: EmulatorState {
                ram: vec![0; 4096],
                pc: 0,
                ireg: 0,
                stack: vec![0; 32],
                delay_timer: 0,
                sound_timer: 0,
                register_bank: vec![0; 16],
                display: vec![false; 32 * 64],
            },
        }
    }

    pub fn step(&mut self, _keys: u16) {
        let opcode = self.fetch();
        if opcode != 0 {
            let op = self.decode(opcode);
            self.execute(op);
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
                0x0EE => todo!(),
                _ => panic!("Opcode {:04X} not supported", opcode),
            },
            0x1 => Ops::Jump(Src::Literal(opcode & 0x0FFF)),
            0x2 => todo!(),
            0x3 => todo!(),
            0x4 => todo!(),
            0x5 => todo!(),
            0x6 => Ops::Add(
                Src::Reg((opcode & 0x0F00 >> 8) as usize),
                Src::Literal(opcode & 0x00FF),
                Src::Literal(0),
            ),
            0x7 => Ops::Add(
                Src::Reg((opcode & 0x0F00 >> 8) as usize),
                Src::Reg((opcode & 0x0F00 >> 8) as usize),
                Src::Literal(opcode & 0x00FF),
            ),
            0x8 => todo!(),
            0x9 => todo!(),
            0xA => Ops::Add(
                Src::IReg,
                Src::Literal(opcode & 0x0FFF >> 8),
                Src::Literal(0),
            ),
            0xB => todo!(),
            0xC => todo!(),
            0xD => Ops::DisplayUpdate(
                Src::Reg(((opcode & 0x0F00) >> 8) as usize),
                Src::Reg(((opcode & 0x00F0) >> 4) as usize),
                Src::Literal(opcode & 0x000F),
            ),
            0xE => todo!(),
            0xF => todo!(),
            _ => unreachable!(),
        }
    }

    fn execute(&mut self, op: Ops) {
        match op {
            Ops::DisplayClear => {
                for i in 0..self.state.display.len() {
                    self.state.display[i] = false;
                }
            }
            Ops::DisplayUpdate(Src::Reg(vx), Src::Reg(vy), Src::Literal(n)) => {
                let px = self.state.register_bank[vx] as usize;
                let py = self.state.register_bank[vy] as usize;
                self.state.register_bank[15] = 0;

                for offset in 0..(n as usize) {
                    let addr = (self.state.ireg as usize) + offset;
                    for b in 0..8 {
                        if self.state.ram[addr] & (1 << b) > 0 {
                            let idx = (py + offset) * 64 + (px + b as usize);
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
            Ops::Jump(Src::Literal(n)) => {
                self.state.pc = n;
            }
            Ops::Jump(_) => panic!("Unsupported Jump: {:?}", op),
            Ops::Add(Src::IReg, Src::Literal(a), Src::Literal(b)) => {
                self.state.ireg = a + b;
            }
            Ops::Add(Src::Reg(vx), Src::Literal(a), Src::Literal(b)) => {
                self.state.register_bank[vx] = ((a + b) % 256) as u8;
            }
            Ops::Add(Src::Reg(vx), Src::Reg(vy), Src::Literal(n)) => {
                self.state.register_bank[vx] =
                    ((self.state.register_bank[vy] as u16 + n) % 256) as u8;
            }
            Ops::Add(_, _, _) => panic!("Unsupported Add: {:?}", op),
        }
    }
}
