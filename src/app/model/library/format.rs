use std::path::Path;
use crate::error::{Result, anyhow, Unknown};
use Format::*;

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Format {
    FLAC,
    MP3,
    WAV,
    OGG,
    Unsupported,
}

impl Format {
    #[inline]
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        match path.as_ref().extension().map(|p| p.to_str()) {
            Some(Some(ext)) => match ext {
                "flac" => Ok(FLAC),
                "mp3" => Ok(MP3),
                "wav" => Ok(WAV),
                "ogg" => Ok(OGG),
                _ => Ok(Unsupported),
            },
            _ => Err(anyhow!(Unknown)),
        }
    }
}
