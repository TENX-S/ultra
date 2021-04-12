use super::prelude::*;
use tui::layout::{Layout, Constraint};
use tui::widgets::Gauge;
use tui::style::Modifier;
use crate::utils::display_duration;
use std::sync::atomic::Ordering;

#[derive(Debug)]
pub struct Timeline {
    pub win_id: u64,
    pub area: Rect,
}

impl Default for Timeline {
    #[inline]
    fn default() -> Self {
        Timeline {
            win_id: 2,
            area: Default::default(),
        }
    }
}

impl View for Timeline {

    #[inline]
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, model: &mut Model, colorscheme: &HashMap<&'static str, Color>) {
        self.area = area;

        let border_style = if self.win_id == model.focus {
            Style::default().fg(colorscheme["focus"])
        } else {
            Style::default().fg(colorscheme["timeline_border"])
        };

        let chunks = Layout::default()
            .constraints(
                [Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(2),].as_ref()
            ).split(area);

        let timeline = Block::default()
            .style(border_style)
            .border_type(BorderType::Thick)
            .borders(Borders::TOP | Borders::BOTTOM);

        f.render_widget(timeline, area);

        let current = display_duration(Some(model.player.elapsed.load(Ordering::Relaxed)));
        let total = if let Some(s) = model.player.current.as_ref() {
            display_duration(s.metadata.duration)
        } else {
            "00:00".to_string()
        };

        let label = format!("{}/{}", current, total);
        let gauge = Gauge::default()
            .block(Block::default())
            .label(label)
            .gauge_style(
                Style::default()
                    .fg(Color::Rgb(168, 216, 185))
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC | Modifier::BOLD),
            )
            .ratio(model.player.ratio().unwrap_or(0.0));

        f.render_widget(gauge, chunks[1]);


    }
}
