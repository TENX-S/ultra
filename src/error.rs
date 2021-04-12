pub use anyhow::{anyhow, Result};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Unknown error.")]
pub struct Unknown;

#[derive(Error, Debug)]
#[error("Non-UTF-8 strings are perfectly valid for this OS.")]
pub struct InvalidUTF8Path;

#[derive(Error, Debug)]
#[error("No preset theme named {0}.")]
pub struct NonexistentPresetTheme(pub String);

#[derive(Error, Debug)]
#[error("The volume value must be between 0 and 100.")]
pub struct InvalidVolume;

#[derive(Error, Debug)]
#[error("Unable to connect to the database!")]
pub struct BrokenConnection;

#[derive(Error, Debug)]
#[error("Value must be an absolute path that exists!")]
pub struct InvalidLocation;

#[derive(Error, Debug)]
#[error("")]
pub struct InvalidColor;
