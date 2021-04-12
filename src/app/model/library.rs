pub mod format;
pub mod song;
pub mod sql;

use crate::{DEBUG, Launch};
use crate::error::{Result, anyhow, BrokenConnection};
use crate::config::Config;
use crate::utils::{setup_logger, get_snapshot, get_last_modified_time,};
use std::sync::atomic::Ordering::Relaxed;
use std::path::PathBuf;
use std::fs;
use std::time::SystemTime;
use sql::*;
use rayon::prelude::*;
use rusqlite::{params, Connection, NO_PARAMS};
use serde::{Deserialize, Serialize};
use song::Song;
use log::{info, trace};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Flag {
    Title,
    Artist,
    Duration,
}

impl Default for Flag {
    #[inline]
    fn default() -> Self {
        Flag::Title
    }
}

#[derive(Debug, Default)]
pub struct Library {
    flag: Flag,
    pub record: Record,
    database: Option<Connection>,
}

#[derive(Debug, Default)]
pub struct Record {
    pub pos: PathBuf,
    pub cache: Vec<Song>,
    modified: Option<SystemTime>,
}

impl Launch for Library {
    #[inline]
    fn bootstrap(&mut self, config: &Config) -> Result<()> {

        if DEBUG.load(Relaxed) {
            info!("Start to bootstrap library.");
        }
        self.record.pos = PathBuf::from(config.lib_pos.as_ref().unwrap());
        self.record.modified = Some(get_last_modified_time(&self.record.pos));

        let db_pos = PathBuf::from(config.db_pos.as_ref().unwrap());
        if !db_pos.exists() {
            if DEBUG.load(Relaxed) {
                trace!("{} doesn't exist, will create it", db_pos.display());
            }
            fs::create_dir_all(&db_pos)?;
        }
        let db = db_pos.join("UltraData.db");
        if !db.exists() {
            if DEBUG.load(Relaxed) {
                trace!("UltraData.db doesn't exist, will create it");
            }
            fs::File::create(&db)?;
        }
        self.database = Some(Connection::open(&db)?);

        self.connection()?.execute(BUILD_RECORD, NO_PARAMS)?;
        self.clean_up()?;

        if let Ok(record) = self.fetch() {
            if DEBUG.load(Relaxed) {
                trace!("GOT RECORD.");
            }
            if self.record.modified == record.modified {
                if DEBUG.load(Relaxed) {
                    trace!("FRESH CACHE.");
                }
                self.record.cache = record.cache;
            } else {
                if DEBUG.load(Relaxed) {
                    trace!("EXPIRED CACHE.");
                }
                self.commit(false)?;
            }
        } else {
            if DEBUG.load(Relaxed) {
                trace!("NO RECORD.");
            }
            self.commit(true)?;
        }
        Ok(())
    }
}

impl Library {

    #[inline]
    pub fn songs(&mut self, flag: Flag, query: Option<String>) -> Result<Vec<Song>> {
        if self.flag != flag {
            self.flag = flag;
            match flag {
                Flag::Title => self.record.cache.par_sort_by_key(|s| {
                    s.metadata
                        .title
                        .as_ref()
                        .map(|s| s.to_owned())
                        .unwrap_or("Unknown".to_owned())
                }),
                Flag::Artist => self.record.cache.par_sort_by_key(|s| {
                    s.metadata
                        .artist
                        .as_ref()
                        .map(|s| s.to_owned())
                        .unwrap_or("Unknown".to_owned())
                }),
                Flag::Duration => self
                    .record
                    .cache
                    .par_sort_by_key(|s| s.metadata.duration.unwrap_or(0)),
            }
        }

        if let Some(query) = query {
            return Ok(self
                .record
                .cache
                .iter()
                .filter(|s| s.row()[..4].join(" ").to_lowercase().contains(&query.to_lowercase()))
                .map(|s| s.clone())
                .collect::<Vec<_>>());
        }

        Ok(self.record.cache.clone())
    }

    #[inline]
    pub fn commit(&mut self, append: bool) -> Result<()> {
        if DEBUG.load(Relaxed) {
            trace!("Commit to database. Append: {}.", append);
        }
        self.sync();
        let conn = self.connection()?;
        if !append {
            conn.execute(
                DELETE_RECORD,
                params![bincode::serialize(&self.record.pos)?],
            )?;
        }
        conn.execute(
            INSERT_RECORD,
            params![
                bincode::serialize(&self.record.pos)?,
                bincode::serialize(&self.record.cache)?,
                bincode::serialize(&self.record.modified)?
            ],
        )?;

        Ok(())
    }

    #[inline]
    fn connection(&self) -> Result<&Connection> {
        if let Some(conn) = self.database.as_ref() {
            Ok(conn)
        } else {
            Err(anyhow!(BrokenConnection))
        }
    }

    #[inline]
    fn sync(&mut self) {
        if DEBUG.load(Relaxed) {
            trace!("Get the latest snapshot.");
        }
        self.record.modified = Some(get_last_modified_time(&self.record.pos));
        self.record.cache = get_snapshot(&self.record.pos)
            .par_iter()
            .filter_map(|p| Song::new(p).ok())
            .collect::<Vec<_>>();
    }

    #[inline]
    fn fetch(&mut self) -> Result<Record> {
        if DEBUG.load(Relaxed) {
            trace!("FETCH cache from database.");
        }
        Ok(self.connection()?.prepare(FETCH_RECORD)?.query_row(
            params![bincode::serialize(&self.record.pos)?],
            |row| {
                Ok(Record {
                    pos: bincode::deserialize(&row.get::<_, Vec<u8>>(0)?).unwrap(),
                    cache: bincode::deserialize(&row.get::<_, Vec<u8>>(1)?).unwrap(),
                    modified: bincode::deserialize(&row.get::<_, Vec<u8>>(2)?).unwrap(),
                })
            },
        )?)
    }

    #[inline]
    fn clean_up(&mut self) -> Result<()> {
        if DEBUG.load(Relaxed) {
            trace!("Clean up the library.")
        }
        let conn = self.connection()?;
        conn.prepare(FETCH_ALL_POS)?
            .query_map(NO_PARAMS, |row| {
                Ok(bincode::deserialize::<PathBuf>(&row.get::<_, Vec<u8>>(0)?).unwrap())
            })?
            .filter_map(Result::ok)
            .for_each(|p| {
                if !p.exists() {
                    conn.execute(DELETE_RECORD, params![bincode::serialize(&p).unwrap()])
                        .unwrap();
                }
            });

        Ok(())
    }
}
