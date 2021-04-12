use crate::error::{Result, anyhow, Unknown, InvalidLocation, InvalidColor};
use crate::SUPPORT_FORMAT;
use rodio::{Decoder, Source};
use std::fs::{self, File};
use std::panic::PanicInfo;
use std::time::SystemTime;
use std::path::{Path, PathBuf};
use std::io::{self, Stdout, BufReader};
use crossterm::style::Print;
use crossterm::ExecutableCommand;
use crossterm::event::DisableMouseCapture;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use tui::style::Color;

#[inline]
pub fn hex_to_rgb(hex: &str) -> Result<Color> {
    if hex.starts_with("#") && hex.len() == 7 {
        if let Some(color) = hex.strip_prefix("#") {
            Ok(Color::Rgb(
                u8::from_str_radix(&color[..2], 16)?,
                u8::from_str_radix(&color[2..4], 16)?,
                u8::from_str_radix(&color[4..], 16)?,
            ))
        } else {
            Err(anyhow!(InvalidColor))
        }
    } else {
        Err(anyhow!(InvalidColor))
    }
}

#[inline]
pub fn get_last_modified_time(path: impl AsRef<Path>) -> SystemTime {
    // Let it to panic if it has to,
    // because if we can't get the last modified time,
    // we can't tell if the cache is fresh or expired.
    fs::metadata(path).unwrap().modified().unwrap()
}

#[inline]
pub fn get_duration(path: impl AsRef<Path>) -> Result<u64> {
    Decoder::new(BufReader::new(File::open(&path)?))?
        .total_duration()
        .map(|t| t.as_secs())
        .ok_or(anyhow!(Unknown))
}

#[inline]
pub fn get_snapshot(path: impl AsRef<Path>) -> Vec<PathBuf> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| {
            !e
                .file_name()
                .to_str()
                .map(|s| s.starts_with("."))
                .unwrap_or(false)
        })
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .filter(|e| e.is_file())
        .filter(|e| {
            if let Some(ext) = e.extension() {
                if let Some(val) = ext.to_str() {
                    SUPPORT_FORMAT.contains(&val)
                } else {
                    false
                }
            } else {
                false
            }
        })
        .collect()
}

#[inline]
pub fn display_duration(duration: Option<u64>) -> String {
    let mut result = "Unknown".to_owned();
    if let Some(duration) = duration {
        let mut hour = 0;
        let mut minutes = 0;
        let seconds;
        if duration > 3600 {
            hour = duration / 3600;
            minutes = (duration - 3600 * hour) / 60;
            seconds = (duration - 3600 * hour) % 60;
        } else if duration > 60 {
            minutes = duration / 60;
            seconds = duration % 60;
        } else {
            seconds = duration;
        }

        if hour != 0 {
            result = format!("{:02}:{:02}:{:02}", hour, minutes, seconds);
            return result;
        }
        if minutes != 0 {
            result = format!("{:02}:{:02}", minutes, seconds);
            return result;
        }
        if seconds != 0 {
            result = format!("00:{:02}", seconds);
            return result;
        }
    }

    result
}

#[inline]
pub fn path_check(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if path.is_absolute() && path.is_dir() {
        Ok(())
    } else {
        Err(anyhow!(InvalidLocation))
    }
}

#[inline]
pub fn panic_hook(panic_info: &PanicInfo<'_>) {
    let mut stdout = io::stdout();

    let msg = match panic_info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match panic_info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };

    let stacktrace: String = format!("{:?}", backtrace::Backtrace::new());

    disable_raw_mode().unwrap();
    stdout
        .execute(DisableMouseCapture)
        .unwrap()
        .execute(LeaveAlternateScreen)
        .unwrap();

    // Print stack trace.  Must be done after!
    stdout
        .execute(Print("Whoops, something went wrong! Please file an issue to https://github.com/TENX/ultra, thank you!\n\n"))
        .unwrap()
        .execute(Print(format!(
            "thread '<unnamed>' panicked at '{}', {}\n\r{}",
            msg,
            panic_info.location().unwrap(),
            stacktrace
        )))
        .unwrap();
}

#[inline]
pub fn setup_logger() -> Result<()> {
    let log_dir = dirs_next::data_dir().unwrap().join("Ultra").join("log");
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} - {} {}",
                record.level(),
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(fern::log_file(
            log_dir.join(format!(
                "{}-ultra_debug.log",
                chrono::Local::now().format("[%Y-%m-%d][%H_%M_%S]")
            )),
        )?)
        .apply()?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_display_duration() {
        assert_eq!(&display_duration(None), "Unknown");
        assert_eq!(&display_duration(Some(0)), "Unknown");
        assert_eq!(&display_duration(Some(1)), "00:01");
        assert_eq!(&display_duration(Some(61)), "01:01");
        assert_eq!(&display_duration(Some(3_599)), "59:59");
        assert_eq!(&display_duration(Some(3_601)), "01:00:01");
        assert_eq!(&display_duration(Some(3_661)), "01:01:01");
        assert_eq!(&display_duration(Some(86_401)), "24:00:01");
        assert_eq!(&display_duration(Some(446_399)), "123:59:59");
    }
}
