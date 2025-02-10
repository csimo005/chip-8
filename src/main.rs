use clap::Parser;
use std::error::Error;

use chip_8::config::Config;
use chip_8::run;

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = Config::parse();
    run(cfg)?;

    Ok(())
}
