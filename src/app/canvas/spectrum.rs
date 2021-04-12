use super::prelude::*;

#[derive(Debug)]
pub struct Spectrum {
    pub win_id: u64,
    pub area: Rect,
}

impl Default for Spectrum {
    #[inline]
    fn default() -> Self {
        Spectrum {
            win_id: 1,
            area: Default::default(),
        }
    }
}

impl View for Spectrum {

    #[inline]
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, model: &mut Model, colorscheme: &HashMap<&'static str, Color>) {
        self.area = area;
        let border_style = if self.win_id == model.focus {
            Style::default().fg(colorscheme["focus"])
        } else {
            Style::default().fg(colorscheme["spectrum_border"])
        };

        let spectrum = Block::default()
            .style(border_style)
            .border_type(BorderType::Thick)
            .borders(Borders::LEFT | Borders::RIGHT);

        f.render_widget(spectrum, area);
    }
}

