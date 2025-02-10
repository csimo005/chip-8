use std::error::Error;

pub struct Config {

}

impl Config {
    pub fn build(_args: Vec<String>) -> Result<Self, Box<dyn Error>> {
        Ok(Self { })
    }
}
