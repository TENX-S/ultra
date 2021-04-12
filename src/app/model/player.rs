use crate::{DEBUG, Launch};
use crate::config::Config;
use crate::error::Result;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::{Relaxed, SeqCst}};
use super::library::song::Song;
use std::io::BufReader;
use rodio::{Sink, OutputStream, Decoder, OutputStreamHandle};
use std::fmt;
use log::{info, trace};

#[derive(Debug)]
pub enum Mode {
    Sequential,
    SingleCycle,
    Random,
}

impl Default for Mode {
    #[inline]
    fn default() -> Self {
        Mode::Sequential
    }
}

#[derive(Default)]
pub struct Player {
    pub mode: Mode,
    volume: u64,
    pub current: Option<Song>,
    pub elapsed: Arc<AtomicU64>,
    paused: Arc<AtomicBool>,
    occupied: Arc<AtomicBool>,
    switched: Arc<AtomicBool>,
    backend: Option<Arc<Sink>>,
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    pub history: Vec<Song>,
}

impl std::fmt::Debug for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player: TODO")
    }
}

impl Launch for Player {
    #[inline]
    fn bootstrap(&mut self, config: &Config) -> Result<()> {
        if DEBUG.load(Relaxed) {
            info!("Start to bootstrap player");
        }

        self.volume = config.volume.unwrap();
        let (_stream, handle) = OutputStream::try_default()?;
        self._stream = Some(_stream);
        let backend = Arc::new(Sink::try_new(&handle)?);
        self.handle = Some(handle);
        backend.set_volume(self.volume as f32 / 100.0);
        self.backend = Some(backend);

        let elapsed = self.elapsed.clone();
        let switched = self.switched.clone();
        thread::spawn(move || {
            loop {
                if switched.load(SeqCst) {
                    switched.store(false, SeqCst);
                    elapsed.store(0, SeqCst);
                }
                thread::sleep(Duration::from_millis(200));
            }
        });

        let paused = self.paused.clone();
        let elapsed = self.elapsed.clone();
        thread::spawn(move || {
            loop {
                if !paused.load(SeqCst) {
                    elapsed.fetch_add(1, SeqCst);
                }
                thread::sleep(Duration::from_secs(1));
            }
        });

        Ok(())
    }
}

impl Player {

    #[inline(always)]
    fn backend(&self) -> &Arc<Sink> {
        self.backend.as_ref().unwrap()
    }

    #[inline]
    pub fn handle(&mut self, song: &Song) -> Result<()> {
        if DEBUG.load(Relaxed) { trace!("Current song is: {:?}", self.current); }
        if !self.history.contains(song) {
            self.history.push(song.clone());
            if DEBUG.load(Relaxed) {
                trace!(
                    "History update! {:#?}",
                    self.history
                        .iter()
                        .map(|s| s.path())
                        .collect::<Vec<_>>()
                )
            }
        }

        if let Some(current) = self.current.as_ref() {
            if current == song {
                if DEBUG.load(Relaxed) { trace!("Press the play button on the same song"); }
                self.play()?;
            } else {
                if DEBUG.load(Relaxed) { trace!("Switch to the song: {:?}", song.path()); }
                self.switch(song)?;
            }
        } else {
            if DEBUG.load(Relaxed) { trace!("Play the first song {:?}", song.path()); }
            self.current = Some(song.clone());
            self.play()?;
        }

        Ok(())
    }

    #[inline]
    pub fn ratio(&self) -> Option<f64> {
        if let Some(current) = self.current.as_ref() {
            if let Some(duration) = current.metadata.duration {
                let val = self.elapsed.load(SeqCst) as f64 / duration as f64;
                if val > 1.0 {
                    Some(0.0)
                } else {
                    Some(val)
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    fn play(&mut self) -> Result<()> {
        if DEBUG.load(Relaxed) {
            trace!("Check the player is occupied or not");
        }
        if self.occupied.load(SeqCst) {
            if DEBUG.load(Relaxed) { trace!("Yes. player is paused: {}.", self.paused.load(SeqCst)) }
            if self.paused.load(SeqCst) {
                self.paused.store(false, SeqCst);
                self.backend().play();
                if DEBUG.load(Relaxed) { trace!("Resume the player."); }
            } else {
                self.paused.store(true, SeqCst);
                self.backend().pause();
                if DEBUG.load(Relaxed) { trace!("Pause the player."); }
            }
        } else {
            self.paused.store(false, SeqCst);
            self.occupied.store(true, SeqCst);
            if DEBUG.load(Relaxed) { trace!("No. The player is occupied from now"); }
            self.backend().append(Decoder::new(BufReader::new(File::open(self.current.as_ref().unwrap().path())?))?);
            if DEBUG.load(Relaxed) { trace!("Append song: {:?} to the queue", self.current); }
            let notify_end = self.backend().clone();
            let occupied = self.occupied.clone();
            thread::spawn(move || {
                loop {
                    if !occupied.load(SeqCst) {
                        if DEBUG.load(Relaxed) { trace!("Thread exits.") }
                        break
                    }
                    if notify_end.is_end() {
                        if DEBUG.load(Relaxed) { trace!("The song is naturally end."); }
                        occupied.store(false, SeqCst);
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            });
        }

        Ok(())
    }

    #[inline]
    fn switch(&mut self, song: &Song) -> Result<()> {
        self.switched.store(true, SeqCst);
        self.backend().stop();
        self.backend = Some(Arc::new(Sink::try_new(self.handle.as_ref().unwrap())?));
        self.current = Some(song.clone());
        self.occupied.store(false, SeqCst);
        if DEBUG.load(Relaxed) { trace!("Cleanup the previous thread."); }
        thread::sleep(Duration::from_millis(200));
        self.play()?;
        Ok(())
    }

    #[inline]
    pub fn increase_volume(&mut self) {
        if self.volume < 99 {
            self.volume += 2;
        } else {
            self.volume = 100;
        }
        self.backend().set_volume(self.volume as f32 / 100.0);
        if DEBUG.load(Relaxed) { trace!("Increase volume. volume: {} .", self.volume) }
    }

    #[inline]
    pub fn decrease_volume(&mut self) {
        if self.volume > 1 {
            self.volume -= 2;
        }  else {
            self.volume = 0;
        }
        self.backend().set_volume(self.volume as f32 / 100.0);
        if DEBUG.load(Relaxed) { trace!("Decrease volume. volume: {} .", self.volume) }
    }

}
