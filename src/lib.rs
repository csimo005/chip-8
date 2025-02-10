use std::error::Error;
use std::{thread, time};

use crate::config::Config;
use crate::emulator::Emulator;
use crate::interface::TUI;
use crate::file_io::read_program;

pub mod config;
pub mod emulator;
pub mod interface;
pub mod file_io;

pub fn run(_cfg: Config) -> Result<(), Box<dyn Error>> {
    let mut tui = TUI::new();
    tui.init_tui();

    let mut ch8 = Emulator::new();
    ch8.load_prog(&read_program("programs/IBM_Logo.ch8")?)?;

    tui.update_tui(&ch8.get_state());
    while tui.is_running() {
        tui.update_keys();
        ch8.step(tui.get_keys());
        tui.update_tui(&ch8.get_state());
        thread::sleep(time::Duration::from_millis(50));
    }

    Ok(())
}
