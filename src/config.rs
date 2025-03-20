#![allow(unused_imports)]
#![allow(dead_code)]

pub struct Config {
    pub sample_rate: u32,
    pub channels: u16,
}

impl Config {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Config {
            sample_rate,
            channels,
        }
    }

    pub fn default() -> Self {
        Config::new(44100, 2)
    }
}
