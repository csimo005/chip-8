use std::error::Error;
use std::{thread, time};

use crate::config::Config;
use crate::emulator::Emulator;
use crate::file_io::read_program;
use crate::interface::TUI;

pub mod config;
pub mod emulator;
pub mod file_io;
pub mod interface;

pub fn run(cfg: Config) -> Result<(), Box<dyn Error>> {
    let mut tui = TUI::new();
    tui.init_tui();

    let mut ch8 = Emulator::new();
    ch8.load_prog(&read_program("programs/IBM_Logo.ch8")?)?;
    tui.update_tui(&ch8.get_state());

    let em_freq: u32 = match cfg.frequency {
        Some(f) => (1000. / f) as u32,
        None => 10,
    };

    let sched: Vec<(u32, Box<dyn Fn(&mut Emulator, &mut TUI)>)> = vec![
        (
            17,
            Box::new(|em, t| {
                t.update_keys();
                t.update_tui(&em.get_state())
            }),
        ),
        (em_freq, Box::new(|em, t| em.step(t.get_keys()))),
    ];
    
    let mut prod = 1;
    for job in sched.iter() {
        prod *= job.0;
    }
    
    let mut cnt: u32 = 0;
    while tui.is_running() {
        for job in sched.iter() {
            if cnt % job.0 == 0 {
                job.1(&mut ch8, &mut tui);
            }
        }

        cnt = (cnt + 1) % prod;
        thread::sleep(time::Duration::from_millis(1));
    }

    Ok(())
}
