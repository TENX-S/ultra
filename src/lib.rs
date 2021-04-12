#![allow(unused, dead_code)]
#![warn(missing_debug_implementations)]
pub mod app;
pub mod cli;
pub mod error;
pub mod utils;
pub mod config;

use config::Config;
use error::Result;
use std::sync::atomic::AtomicBool;
use std::path::PathBuf;
use lazy_static::lazy_static;

pub const SUPPORT_FORMAT: [&str; 4] = ["flac", "mp3", "wav", "ogg"];

static DEBUG: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref CONFIG_PATH: PathBuf = dirs_next::config_dir().unwrap().join("Ultra").join("ultra.toml");
}

pub trait Launch {
    fn bootstrap(&mut self, config: &Config) -> Result<()>;
}
