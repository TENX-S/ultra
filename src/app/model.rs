pub mod player;
pub mod library;

use player::Player;
use library::{Library, Flag};
use crate::Launch;
use crate::error::Result;
use crate::config::Config;
use crate::app::model::library::song::Song;
use rand::prelude::*;
use tui::widgets::{TableState, ListState};
use crate::app::model::player::Mode;
use log::{info, trace};

#[derive(Debug, Default)]
pub struct Model {
    pub focus: u64,
    pub flag: Flag,
    pub query: String,
    pub player: Player,
    pub topline: usize,
    pub baseline: usize,
    pub library: Library,
    pub offset: Option<usize>,
    pub board_state: TableState,
    pub songs: Vec<Vec<String>>,
    pub current_play_idx: Option<usize>,
}

impl Launch for Model {
    fn bootstrap(&mut self, config: &Config) -> Result<()> {
        self.player.bootstrap(config)?;
        self.library.bootstrap(config)?;
        self.sync_headers()?;
        Ok(())
    }
}

impl Model {

    #[inline]
    pub fn select_next_song(&mut self) {
        self.offset = match self.board_state.selected() {
            Some(i) => {
                if i >= self.songs.len() - 1 {
                    Some(0)
                } else {
                    Some(i + 1)
                }
            }
            None => Some(0),
        };
        self.board_state.select(self.offset);
    }

    #[inline]
    pub fn select_previous_song(&mut self) {
        self.offset = match self.board_state.selected() {
            Some(i) => {
                if i == 0 {
                    Some(self.songs.len() - 1)
                } else {
                    Some(i - 1)
                }
            }
            None => Some(0),
        };
        self.board_state.select(self.offset);
    }

    #[inline]
    pub fn open_search(&mut self) {
        self.focus = 3;
    }

    #[inline]
    pub fn close_search(&mut self) {
        self.focus = 0;
        self.query.clear();
        self.unselect_board();
    }

    #[inline]
    pub fn select_board(&mut self, pos: usize) {
        self.board_state.select(Some(pos));
        self.offset = Some(pos);
    }

    #[inline]
    pub fn unselect_board(&mut self) {
        self.board_state.select(None);
        self.offset = None;
    }

    pub fn unselect_spectrum(&mut self) {
        // TODO
    }

    #[inline]
    pub fn flag(&mut self, flag: Flag) {
        self.flag = flag;
    }

    #[inline]
    pub fn sync_headers(&mut self) -> Result<()> {
        let mut idx = 1;
        let query = self.query().clone();
        self.songs = self.library
            .songs(self.flag, query)?
            .iter()
            .map(|s| {
                let mut raw_row = s.row();
                raw_row.insert(0, idx.to_string());
                idx += 1;
                raw_row
            })
            .collect();

        Ok(())
    }

    #[inline]
    pub fn query(&self) -> Option<String> {
        if !self.query.is_empty() && self.focus == 3 {
            Some(self.query.clone())
        } else {
            None
        }
    }



    // #[inline]
    // pub fn next_song(&self) -> usize {
    //     match self.player.mode {
    //         Mode::Sequential => {
    //             if let Some(i) = self.current_play_idx {
    //                 if i >= self.songs
    //             }
    //         }
    //         Mode::SingleCycle => {}
    //         Mode::Random => {}
    //     }
    // }

}
