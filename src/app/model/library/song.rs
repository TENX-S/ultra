use super::format::Format::{self, *};
use crate::error::{Result, anyhow, Unknown};
use crate::utils::{display_duration, get_duration};
use id3::Tag as MP3Tag;
use metaflac::Tag as FLACTag;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Song {
    pos: PathBuf,
    f_name: OsString,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<u64>,
}

impl Song {
    #[inline]
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut pos = path.as_ref().to_path_buf();
        let f_name;
        if let Some(val) = pos.file_name() {
            f_name = val.to_os_string();
        } else {
            return Err(anyhow!(Unknown)); // TODO
        }
        pos.pop();

        let metadata = Metadata::new(path)?;
        Ok(Song {
            pos,
            f_name,
            metadata,
        })
    }

    #[inline(always)]
    pub fn path(&self) -> PathBuf {
        self.pos.join(&self.f_name)
    }

    #[inline]
    pub fn row(&self) -> Vec<String> {
        vec![
            self.metadata.title.clone().unwrap_or_else(|| {
                self.path()
                    .file_stem()
                    .map(|p| p.to_str().unwrap())
                    .unwrap_or("Unknown")
                    .to_owned()
            }),
            self.metadata
                .album
                .clone()
                .unwrap_or_else(|| "Unknown".to_owned()),
            self.metadata
                .artist
                .clone()
                .unwrap_or_else(|| "Unknown".to_owned()),
            display_duration(self.metadata.duration),
        ]
    }
}

impl Metadata {
    #[inline]
    fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut title = None;
        let mut artist = None;
        let mut album = None;
        let mut duration = None;

        match Format::new(&path)? {
            FLAC => {
                let tag = FLACTag::read_from_path(&path)?;
                if let Some(vbscmt) = tag.vorbis_comments() {
                    title = vbscmt.title().map(|v| v.join(" "));
                    artist = vbscmt.artist().map(|v| v.join(" "));
                    album = vbscmt.album().map(|v| v.join(" "));
                    duration = get_duration(&path).ok();
                }
            }
            MP3 => {
                let tag = MP3Tag::read_from_path(&path)?;
                title = tag.title().map(str::to_string);
                artist = tag.artist().map(str::to_string);
                album = tag.album().map(str::to_string);
                duration = tag.duration().map(|t| t as u64);
                if duration.is_none() {
                    duration = get_duration(&path).ok();
                }
            }
            WAV => {
                // TODO
            }
            OGG => {
                // TODO
            }
            Unsupported => {
                // TODO
            }
        }

        Ok(Metadata {
            title,
            artist,
            album,
            duration,
        })
    }
}
