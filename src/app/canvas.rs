pub mod board;
pub mod prelude;
pub mod timeline;
pub mod spectrum; // TODO

use board::Board;
use timeline::Timeline;
use spectrum::Spectrum;
use tui::{
    Frame,
    style::{Color, Style},
    backend::Backend,
    widgets::{
        Block, Borders, BorderType
    },
    layout::{
        Rect, Layout,
        Constraint, Direction,
    },
};
use super::model::Model;
use crate::Launch;
use crate::config::{Config, Theme};
use crate::error::{Result, anyhow, NonexistentPresetTheme};
use std::collections::HashMap;

pub trait View {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, model: &mut Model, colorscheme: &HashMap<&'static str, Color>);
}

#[derive(Debug, Default)]
pub struct Canvas {
    pub board: Board,
    pub timeline: Timeline,
    pub spectrum: Spectrum,
    overlay_background: bool,
    pub colorscheme: HashMap<&'static str, Color>,
}

impl Launch for Canvas {
    fn bootstrap(&mut self, config: &Config) -> Result<()> {

        self.overlay_background = config.overlay_background.unwrap();

        if let Some(c) = config.theme.as_ref().unwrap().preset.as_ref() {
            if c == "Dark" { self.colorscheme = Theme::dark().colorscheme()?; }
            if c == "Light" { self.colorscheme = Theme::light().colorscheme()?; }
        } else {
            self.colorscheme = config.theme.as_ref().unwrap().colorscheme()?;
        }

        Ok(())
    }
}

impl Canvas {
    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, model: &mut Model) {

        let screen = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .split(f.size())[0];

        let canvas = if self.overlay_background {
            Block::default()
                .style(Style::default().bg(self.colorscheme["background"]))
        } else {
            Block::default()
        };

        f.render_widget(canvas, screen);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
            .split(screen);

        let up = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(chunks[0]);
        self.board.draw(f, up[1], model, &self.colorscheme);
        self.timeline.draw(f, chunks[1], model, &self.colorscheme);
        self.spectrum.draw(f, up[0], model, &self.colorscheme);
    }
}
