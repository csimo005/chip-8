use crate::emulator::EmulatorState;
use std::error::Error;
use std::fs;

pub fn read_program(fname: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(fs::read(fname)?)
}

pub fn write_program(_fname: &str, _prog: &[u8]) {
    todo!()
}

pub fn read_state(_fname: &str) -> EmulatorState {
    todo!()
}

pub fn write_state(_fname: &str, _state: &EmulatorState) {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_program() {
        let prog = read_program("programs/test_opcode.ch8").unwrap();
        assert_eq!(prog.len(), 478);

        assert_eq!(prog[0], 0x12);
        assert_eq!(prog[1], 0x4E);
        assert_eq!(prog[2], 0xEA);
        assert_eq!(prog[3], 0xAC);

        assert_eq!(prog[prog.len() - 4], 0x12);
        assert_eq!(prog[prog.len() - 3], 0x48);
        assert_eq!(prog[prog.len() - 2], 0x13);
        assert_eq!(prog[prog.len() - 1], 0xDC);
    }
}
