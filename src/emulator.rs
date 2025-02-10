use std::error::Error;

enum Ops {
    
}

pub struct EmulatorState { // Going to seperate state out, for use in file io
    pub ram: Vec<u8>,          // and anticipating emulator will need extra stuff
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
            }
        }
    }

    pub fn step(&mut self, _keys: u16) {
        let _opcode = self.fetch();
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
            self.state.ram[i] = prog[i];
        }
        Ok(())
    }

    fn fetch(&mut self) -> u16 {
        let opcode: u16 = ((self.state.ram[self.state.pc as usize] as u16) << 8) + (self.state.ram[(self.state.pc as usize) + 1] as u16);
        self.state.pc += 2;
        opcode
    }
}
