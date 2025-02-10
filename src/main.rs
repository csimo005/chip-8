use std::env;
use std::error::Error;

use chip_8::config::Config;
use chip_8::run;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let cfg = Config::build(args)?;
    run(cfg)?;

    Ok(())
}
