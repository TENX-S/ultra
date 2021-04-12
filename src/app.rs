pub mod canvas;
pub mod model;

use crate::{DEBUG, Launch, CONFIG_PATH};
use crate::cli::CLI;
use crate::config::Config;
use crate::app::model::{library::Flag, player::Mode};
use crate::utils::{panic_hook, setup_logger};
use crate::app::canvas::View;
use crate::error::{Result, anyhow};

use std::panic;
use std::thread;
use std::time::{Instant, Duration};
use std::path::{PathBuf, Path};
use std::io::{self, Stdout};
use std::ops::RangeInclusive;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::sync::atomic::{AtomicBool, Ordering::{Relaxed, SeqCst}};
use canvas::Canvas;
use model::Model;
use crossterm::ExecutableCommand;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::event::{self, EnableMouseCapture, DisableMouseCapture, Event as TermEvent, KeyCode, KeyEvent, MouseEvent, MouseButton, KeyModifiers};
use tui::{Frame, Terminal};
use tui::backend::{CrosstermBackend, Backend};
use unicode_width::UnicodeWidthStr;
use notify::{Watcher, DebouncedEvent};
use log::{info, trace};

macro_rules! click {
    ($x:expr, $y:expr, $view:expr, $model:expr) => {

        if RangeInclusive::new($view.area.left(), $view.area.right()).contains(&$x)
        && RangeInclusive::new($view.area.top(), $view.area.bottom()).contains(&$y) {
            $model.focus = $view.win_id;
        }
    };
}

macro_rules! trace_value {
    ($offset:expr, $topline:expr, $baseline:expr) => {
        trace!("-- offset: {}", $offset);
        trace!("-- topline: {}", $topline);
        trace!("-- baseline: {}", $baseline);
    };
}

#[derive(Debug)]
pub enum Event {
    LibraryChanged,
    ConfigUpdated,
    UserInput(TermEvent),
}

#[derive(Default, Debug)]
pub struct App {
    model: Model,
    canvas: Canvas,
    terminated: bool,
}

impl Launch for App {

    fn bootstrap(&mut self, config: &Config) -> Result<()> {
        if config.debug.unwrap() {
            DEBUG.store(true, Relaxed);
            setup_logger()?;
        }

        self.model.bootstrap(config)?;
        self.canvas.bootstrap(config)?;

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout
            .execute(EnableMouseCapture)?
            .execute(EnterAlternateScreen)?;

        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

        terminal.clear()?;
        terminal.hide_cursor()?;

        let (event_tx, event_rx) = mpsc::channel();
        let rc_event_rx = Rc::new(event_rx);

        let (w_tx, w_rx) = mpsc::channel();
        let mut watcher = notify::watcher(w_tx, Duration::from_millis(100))?;
        watcher.watch(r"/Users/tenx/Music", notify::RecursiveMode::Recursive)?;
        watcher.watch(&*CONFIG_PATH, notify::RecursiveMode::NonRecursive)?;

        let config = config.clone();
        let mut timer = Instant::now();
        let _event_listener = thread::spawn(move || loop {

            if let Ok(poll) = event::poll(Duration::from_millis(30)) {
                if poll {
                    if let Ok(term_event) = event::read() {
                        if Instant::now().duration_since(timer).as_millis() >= 30 {
                            event_tx.send(Event::UserInput(term_event)).unwrap();
                            timer = Instant::now();
                        }
                    }
                }
            }

            if let Ok(debounced_event) = w_rx.try_recv() {
                match debounced_event {
                    DebouncedEvent::Write(p) => {
                        if p == CONFIG_PATH.to_path_buf() {
                            event_tx.send(Event::ConfigUpdated).unwrap();
                        }
                    }
                    DebouncedEvent::Create(p) |
                    DebouncedEvent::Remove(p) |
                    DebouncedEvent::Rename(p, _) => {
                        if p.ancestors().find(|p| p.to_path_buf() == PathBuf::from(config.lib_pos.as_ref().unwrap())).is_some() {
                            event_tx.send(Event::LibraryChanged).unwrap();
                        }
                    }
                    DebouncedEvent::Rescan => {
                        event_tx.send(Event::LibraryChanged).unwrap();
                    }
                    _ => {}
                }

            }

        });

        while !self.terminated {
            terminal.draw(|f| self.draw(f))?;
            self.handle_event(rc_event_rx.clone())?;
            thread::sleep(Duration::from_millis(50));
        }

        disable_raw_mode()?;
        terminal
            .backend_mut()
            .execute(DisableMouseCapture)?
            .execute(LeaveAlternateScreen)?
            .show_cursor()?;

        Ok(())
    }

}

impl App {

    #[inline]
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>) {
        self.canvas.draw(f, &mut self.model);
    }

    #[inline]
    fn handle_event(&mut self, event_rx: Rc<Receiver<Event>>) -> Result<()> {
        self.sync_boundary();
        if let Ok(event) = event_rx.try_recv() {
            // if DEBUG.load(Relaxed) { trace!("RECEIVE EVENT: {:#?}", event) }
            match event {
                Event::ConfigUpdated => {
                    // TODO
                }
                Event::LibraryChanged => {
                    self.model.library.commit(false)?;
                    self.model.sync_headers()?;
                }
                Event::UserInput(term_event) => {
                    match term_event {
                        TermEvent::Key(key_event) => self.handle_key(key_event)?,
                        TermEvent::Mouse(mouse_event) => self.handle_mouse(mouse_event)?,

                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    #[inline]
    fn handle_key(&mut self, event: KeyEvent) -> Result<()> {
        if event == KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL) {
            self.terminated = true;
        }
        match event.code {
            KeyCode::Char(c) => self.on_char(c)?,
            KeyCode::Esc => self.on_esc(),
            KeyCode::Backspace => self.on_backspace(),
            KeyCode::Up => self.on_up(),
            KeyCode::Down => self.on_down(),
            KeyCode::Left => self.on_left(),
            KeyCode::Right => self.on_right(),
            KeyCode::Enter => self.on_enter()?,
            KeyCode::Tab => self.on_tab(),
            KeyCode::F(n) => self.on_f(n),
            _ => {}
        }
        Ok(())
    }

    #[inline]
    fn handle_mouse(&mut self, event: MouseEvent) -> Result<()> {
        match event {
            MouseEvent::ScrollDown(_, _, _) => self.on_scroll_down(),
            MouseEvent::ScrollUp(_, _, _) => self.on_scroll_up(),
            MouseEvent::Down(button, x, y, _) => {
                if matches!(button, MouseButton::Left) {
                    self.on_click(x, y);
                }
            }
            _ => {}
        }
        Ok(())
    }

    #[inline]
    fn on_char(&mut self, c: char) -> Result<()> {
        if c == 'q' && self.model.focus != 3 {
            self.terminated = true;
        }

        match self.model.focus {
            0 => {
                match c {
                    '/' => self.model.open_search(),
                    't' => self.model.flag(Flag::Title),
                    'a' => self.model.flag(Flag::Artist),
                    'd' => self.model.flag(Flag::Duration),
                     _  => {}
                }
            }
            3 => {
                if !c.is_control() && self.model.query.width() <= 64 {
                    self.model.query.push(c);
                    self.model.unselect_board();
                }
            }

            _ => {}
        }

        Ok(())
    }

    #[inline]
    fn on_esc(&mut self) {
        match self.model.focus {
            0 => self.model.unselect_board(),
            1 => self.model.unselect_spectrum(),
            3 => self.model.close_search(),
            _ => {}
        }
    }

    #[inline]
    fn on_backspace(&mut self) {
        match self.model.focus {
            3 => {
                self.model.query.pop();
                self.model.unselect_board();
                if self.model.query.is_empty() {
                    self.model.close_search();
                }
            },
            _ => {}
        }
    }

    #[inline]
    fn on_up(&mut self) {
        match self.model.focus {
            0 | 3 => self.model.select_previous_song(),
            1 => {}
            2 => self.model.player.increase_volume(),
            _ => {}
        }
    }

    #[inline]
    fn on_down(&mut self) {
        match self.model.focus {
            0 | 3 => self.model.select_next_song(),
            1 => {}
            2 => self.model.player.decrease_volume(),

            _ => {}
        }
    }

    #[inline]
    fn on_left(&mut self) {
        match self.model.focus {
            // 2 => self.model.
            _ => {}
        }
    }

    #[inline]
    fn on_right(&mut self) {
        match self.model.focus {
            // 2 => self.model.player.play(&self.model.next_song().unwrap()),
            _ => {}
        }
    }

    #[inline]
    fn on_enter(&mut self) -> Result<()> {
        match self.model.focus {
            0 | 3 => {
                if let Some(target) = self.model.offset {
                    self.model.player.handle(&self.model.library.record.cache[target])?;
                }
            }
            1 => {}
            _ => {}
        }
        Ok(())
    }

    #[inline]
    fn on_tab(&mut self) {
        match self.model.focus {
            0 => self.model.focus = 2,
            1 => self.model.focus = 0,
            2 => self.model.focus = 1,
            _ => {}
        }
    }

    #[inline]
    fn on_f(&mut self, n: u8) {
        match n {
            1 => self.model.player.mode = Mode::Sequential,
            2 => self.model.player.mode = Mode::Random,
            3 => self.model.player.mode = Mode::SingleCycle,
            _ => {}
        }
    }

    #[inline]
    fn on_scroll_up(&mut self) {
        match self.model.focus {
            0 | 3 => self.model.select_previous_song(),
            _ => {}
        }
    }

    #[inline]
    fn on_scroll_down(&mut self) {
        match self.model.focus {
            0 | 3 => self.model.select_next_song(),
            _ => {}
        }
    }

    #[inline]
    fn on_click(&mut self, x: u16, y: u16) {
        if self.model.focus != 3 {
            click!(x, y, self.canvas.board, self.model);
            click!(x, y, self.canvas.spectrum, self.model);
            click!(x, y, self.canvas.timeline, self.model);
            if self.model.focus == 0 && y > 1 {
                if let Some(offset) = self.model.offset {
                    self.model.select_board(y as usize - 2 + self.model.topline);
                } else {
                    self.model.select_board(y as usize - 2);
                }
            }
        }
    }

    #[inline]
    fn sync_boundary(&mut self) {
        if self.model.focus == 0 {
            let song_height = self.canvas.board.area.height as usize - 2;
            let song_nums = self.model.library.record.cache.len();

            if let Some(offset) = self.model.offset {
                if offset <= song_height && self.model.topline == 0 {
                    self.model.baseline = song_height - 1;
                    return;
                }

                if offset == song_nums - 1 {
                    self.model.baseline = song_nums - 1;
                    self.model.topline = self.model.baseline - (song_height - 1);
                    return;
                }

                if offset > self.model.baseline {
                    self.model.baseline = offset;
                    self.model.topline = self.model.baseline - (song_height - 1);
                    return;
                }

                if offset < self.model.topline {
                    self.model.topline = offset;
                    self.model.baseline = self.model.baseline + song_height - 1;
                    return;
                }
            }

        }
    }

}
