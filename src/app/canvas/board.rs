use super::prelude::*;
use unicode_width::UnicodeWidthStr;
use tui::layout::{Layout, Constraint, Direction};
use tui::widgets::{Paragraph, Table, Cell, Row};
use tui::style::Modifier;

#[derive(Debug)]
pub struct Search {
    pub win_id: u64,
    pub area: Rect,
}

impl Default for Search {
    #[inline]
    fn default() -> Self {
        Search {
            win_id: 3,
            area: Default::default(),
        }
    }
}

impl View for Search {

    #[inline]
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, model: &mut Model, colorscheme: &HashMap<&'static str, Color>) {
        self.area = area;

        let text = vec![
            Spans::from(
                Span::styled(model.query.as_str(), Style::default().fg(colorscheme["query"]))
            )
        ];
        f.set_cursor(area.x + model.query.width() as u16 + 1, area.y + 1);

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .style(Style::default().fg(colorscheme["search_border"]))
                    .border_type(BorderType::Thick)
                    .borders(Borders::ALL)
            );
        f.render_widget(paragraph, area);
    }

}

#[derive(Debug, Default)]
pub struct Board {
    pub win_id: u64,
    pub area: Rect,
    pub search: Search,
}

impl View for Board {
    #[inline]
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, model: &mut Model, colorscheme: &HashMap<&'static str, Color>) {
        self.area = area;

        let border_style = if self.win_id == model.focus {
            Style::default().fg(colorscheme["focus"])
        } else {
            Style::default().fg(colorscheme["board_border"])
        };
        let selected_style = Style::default().fg(colorscheme["board_selected"]).add_modifier(Modifier::REVERSED);
        let unselected_style = Style::default().fg(colorscheme["board_unselected"]);
        let header_style = Style::default().fg(colorscheme["board_header"]).add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

        let rows = model.songs.iter().map(|item| {
            let cells = item.iter().map(|c| Cell::from(c.clone()));
            Row::new(cells).height(1)
        });

        let lib = Block::default()
            .border_style(border_style)
            .border_type(BorderType::Thick)
            .borders(Borders::LEFT | Borders::RIGHT);

        let header_cells = ["No.", "Title", "Artist", "Album", "Duration"]
            .iter()
            .map(|n| Cell::from(*n));

        let header = Row::new(header_cells)
            .style(header_style)
            .height(1)
            .bottom_margin(1);

        let board = Table::new(rows)
            .header(header)
            .block(lib)
            .style(unselected_style)
            .highlight_style(selected_style)
            .highlight_symbol("> ")
            .widths(&[
                Constraint::Percentage(5),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(25),
                Constraint::Percentage(10),
            ]);

        if model.focus == 3 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                .split(area);
            f.render_stateful_widget(board, chunks[0], &mut model.board_state);
            self.search.draw(f, chunks[1], model, colorscheme);
        } else {
            f.render_stateful_widget(board, area, &mut model.board_state);
        }

    }
}
