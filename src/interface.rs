use std::io::{stdin, stdout, Stdout, Write};

use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::style;

use device_query::{DeviceQuery, DeviceState, Keycode};

use crate::emulator::EmulatorState;

pub struct TUI {
    stdout: RawTerminal<Stdout>,
    display: Vec<bool>,
    prog: Vec<u8>,
    pc: Option<u16>,
    prog_offset: Option<u16>,
    keys: Option<u16>,
    running: bool,
}

impl TUI {
    pub fn new() -> Self {
        let stdout = stdout().into_raw_mode().unwrap();

        Self {
            stdout: stdout,
            display: vec![false; 32 * 64],
            prog: vec![0; 4096],
            pc: None,
            prog_offset: None,
            keys: None,
            running: true,
        }
    }

    pub fn init_tui(&mut self) {
        write!(
            self.stdout,
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            termion::cursor::Hide
        )
        .unwrap();
        write!(self.stdout, "┌────────────────────────────Display─────────────────────────────┬──────────────────────────RAM───────────────────────────┐\r\n").unwrap();
        for i in 0..32 {
            write!(
                self.stdout,
                "│                                                                │"
            )
            .unwrap();
            write!(
                self.stdout,
                " {:04X}:  ....  ....  ....  ....  ....  ....  ....  ....  │\r\n",
                i * 16
            )
            .unwrap();
        }
        write!(self.stdout, "├────┬─────────────────────────────────────────────────────────┬─┴──────────┬──────────Reg─────────┬─────────Stack────────┤\r\n").unwrap();
        write!(self.stdout, "│123C│                                                         │   PC: .... │....  ....  ....  ....│....  ....  ....  ....│\r\n").unwrap();
        write!(self.stdout, "│456D│                                                         │ IREG: .... │....  ....  ....  ....│....  ....  ....  ....│\r\n").unwrap();
        write!(self.stdout, "│789E│                                                         │Delay:   .. │....  ....  ....  ....│....  ....  ....  ....│\r\n").unwrap();
        write!(self.stdout, "│A0BF│                                                         │Sound:   .. │....  ....  ....  ....│....  ....  ....  ....│\r\n").unwrap();
        write!(self.stdout, "└────┘                                                         └────────────┴──────────────────────┴──────────────────────┘\r\n").unwrap();

        self.stdout.flush().unwrap();
    }

    pub fn update_tui(&mut self, state: &EmulatorState) {
        self.draw_display(&state.display);
        self.draw_keypad();
        self.draw_program(&state.ram, state.pc);
        self.draw_values(state.pc, state.ireg, state.delay_timer, state.sound_timer);
        self.draw_registers(&state.register_bank);

        self.stdout.flush().unwrap();
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn update_keys(&mut self) {
        let device_state = DeviceState::new();
        let keys: Vec<Keycode> = device_state.get_keys();

        if keys.contains(&Keycode::LControl) || keys.contains(&Keycode::RControl) {
            if keys.contains(&Keycode::Q) {
                self.running = false;
            }
        } else {
            let keycodes: Vec<_> = vec![
                Keycode::V,
                Keycode::C,
                Keycode::X,
                Keycode::Z,
                Keycode::F,
                Keycode::D,
                Keycode::S,
                Keycode::A,
                Keycode::R,
                Keycode::E,
                Keycode::W,
                Keycode::Q,
                Keycode::Key4,
                Keycode::Key3,
                Keycode::Key2,
                Keycode::Key1,
            ];

            let mut k = 0;
            for i in 0..16 {
                k *= 2;
                if keys.contains(&keycodes[i]) {
                    k += 1;
                }
            }

            self.keys = Some(k);
        }
    }

    pub fn get_keys(&self) -> u16 {
        self.keys.unwrap()
    }

    fn draw_display(&mut self, display: &Vec<bool>) {
        for r in 0..32 {
            for c in 0..64 {
                let idx = (r * 64 + c) as usize;
                if self.display[idx] != display[idx] {
                    write!(
                        self.stdout,
                        "{}{}",
                        termion::cursor::Goto(c + 2, r + 2),
                        match display[(r * 64 + c) as usize] {
                            true => "█",
                            false => " ",
                        }
                    )
                    .unwrap();
                    self.display[idx] = display[idx];
                }
            }
        }
    }

    fn draw_keypad(&mut self) {
        let syms: Vec<char> = vec![
            '1', '2', '3', 'C', '4', '5', '6', 'D', '7', '8', '9', 'E', 'A', '0', 'B', 'F',
        ];
        for i in 0..syms.len() {
            let r = 35 + ((i / 4) as u16);
            let c = 2 + ((i % 4) as u16);
            if let Some(k) = self.keys {
                if (k & (1 << i)) > 0 {
                    write!(
                        self.stdout,
                        "{}{}{}{}",
                        termion::cursor::Goto(c, r),
                        style::Invert,
                        syms[i],
                        style::NoInvert
                    )
                    .unwrap();
                } else {
                    write!(self.stdout, "{}{}", termion::cursor::Goto(c, r), syms[i]).unwrap();
                }
            }
        }
    }

    fn draw_program(&mut self, prog: &[u8], pc: u16) {
        for i in 0..2048 {
            if self.prog[2 * i] != prog[2 * i] || self.prog[2 * i + 1] != prog[2 * i + 1] {
                self.prog[2 * i] = prog[2 * i];
                self.prog[2 * i + 1] = prog[2 * i + 1];

                if let Some(offset) = self.prog_offset {
                    let line = ((2 * i) / 16) as u16;
                    if offset <= line && line < (offset + 32) {
                        // lines rendered
                        let cmd = ((self.prog[2 * i] as u16) << 8) + (self.prog[2 * i + 1] as u16);
                        write!(
                            self.stdout,
                            "{}{:04X}",
                            termion::cursor::Goto(
                                (((2 * i) % 16) * 3 + 75) as u16,
                                ((line - self.prog_offset.unwrap()) + 2) as u16
                            ),
                            cmd
                        )
                        .unwrap();
                    }
                }
            }
        }

        if let Some(offset) = self.prog_offset {
            let line = pc / 16;

            if line < offset || line >= (offset + 32) {
                // PC out of disp range
                if line < offset {
                    self.prog_offset = Some(line);
                } else {
                    self.prog_offset = Some(offset + line - (offset + 31));
                }

                for l in 0..32 {
                    write!(
                        self.stdout,
                        "{}{:04X}",
                        termion::cursor::Goto(68, (l + 2) as u16),
                        self.prog_offset.unwrap() + l
                    )
                    .unwrap();

                    for c in 0..8 {
                        let pc = (self.prog_offset.unwrap() + l) * 16 + 2 * c;
                        let cmd = ((self.prog[pc as usize] as u16) << 8)
                            + (self.prog[(pc + 1) as usize] as u16);

                        write!(
                            self.stdout,
                            "{}{:04X}",
                            termion::cursor::Goto((6 * c + 75) as u16, (l + 2) as u16),
                            cmd
                        )
                        .unwrap();
                    }
                }
            }
        } else {
            self.prog_offset = Some(pc / 16);
            for l in 0..32 {
                write!(
                    self.stdout,
                    "{}{:04X}",
                    termion::cursor::Goto(68, (l + 2) as u16),
                    (self.prog_offset.unwrap() + l) * 16,
                )
                .unwrap();

                for c in 0..8 {
                    let pc = (self.prog_offset.unwrap() + l) * 16 + 2 * c;
                    let cmd = ((self.prog[pc as usize] as u16) << 8)
                        + (self.prog[(pc + 1) as usize] as u16);

                    write!(
                        self.stdout,
                        "{}{:04X}",
                        termion::cursor::Goto((6 * c + 75) as u16, (l + 2) as u16),
                        cmd
                    )
                    .unwrap();
                }
            }
        }

        if let Some(old_pc) = self.pc {
            let line = self.pc.unwrap() / 16;

            if self.prog_offset.unwrap() <= line || line < (self.prog_offset.unwrap() + 32) {
                let col = (old_pc % 16) * 3 + 75;
                let cmd = ((self.prog[old_pc as usize] as u16) << 8)
                    + (self.prog[(old_pc + 1) as usize] as u16);
                write!(
                    self.stdout,
                    "{}{:04X}",
                    termion::cursor::Goto(
                        col as u16,
                        ((line - self.prog_offset.unwrap()) + 2) as u16
                    ),
                    cmd
                )
                .unwrap();
            }
        }

        self.pc = Some(pc);
        let line = self.pc.unwrap() / 16;
        let col = (self.pc.unwrap() % 16) * 3 + 75;
        let cmd = ((self.prog[self.pc.unwrap() as usize] as u16) << 8)
            + (prog[(self.pc.unwrap() + 1) as usize] as u16);
        write!(
            self.stdout,
            "{}{}{:04X}{}",
            termion::cursor::Goto(col as u16, ((line - self.prog_offset.unwrap()) + 2) as u16),
            style::Invert,
            cmd,
            style::NoInvert
        )
        .unwrap();
    }

    fn draw_values(&mut self, pc: u16, ireg: u16, delay: u8, sound: u8) {
        write!(self.stdout, "{}{:04X}", termion::cursor::Goto(72, 35), pc).unwrap();
        write!(self.stdout, "{}{:04X}", termion::cursor::Goto(72, 36), ireg).unwrap();
        write!(
            self.stdout,
            "{}{:02X}",
            termion::cursor::Goto(74, 37),
            delay
        )
        .unwrap();
        write!(
            self.stdout,
            "{}{:02X}",
            termion::cursor::Goto(74, 38),
            sound
        )
        .unwrap();
    }

    fn draw_registers(&mut self, register_bank: &Vec<u8>) {
        for i in 0..16 {
            let row = (i / 4) + 35;
            let col = (i % 4) * 6 + 78;

            write!(
                self.stdout,
                "{}{:04X}",
                termion::cursor::Goto(col, row),
                register_bank[i as usize]
            )
            .unwrap();
        }
    }
}

impl Drop for TUI {
    fn drop(&mut self) {
        write!(
            self.stdout,
            "{}{}",
            termion::cursor::Goto(1, 40),
            termion::cursor::Show
        )
        .unwrap();
        self.stdout.flush().unwrap();

        let stdin = stdin();
        for _ in stdin.keys() {
            break;
        }
    }
}
