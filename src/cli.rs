use std::path::PathBuf;
use crate::config::{Config, Theme};
use clap::{self, App, Arg, ArgMatches};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct CLI<'a> {
    args: ArgMatches<'a>,
}

impl CLI<'_> {
    #[inline]
    pub fn new() -> Self {
        let path_check = |path: String| {
            let path = PathBuf::from(path);
            if path.is_absolute() && path.is_dir() {
                Ok(())
            } else {
                Err("Value must be an absolute path that exists!".into())
            }
        };

        CLI {
            args: App::new(clap::crate_name!())
                .author(clap::crate_authors!())
                .version(clap::crate_version!())
                .about(clap::crate_description!())
                .arg(
                    Arg::with_name("overlay-background")
                        .value_name("BOOL")
                        .short("b")
                        .long("overlay-background")
                        .help("Overlay the background or not.")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("preset-theme")
                        .value_name("NAME")
                        .short("t")
                        .long("preset-theme")
                        .help("Set preset theme. Available values: 'dark', 'light'.")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("INPUT")
                        .value_name("PATH")
                        .index(1)
                        .help("Set the library to open.")
                        .validator(path_check)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("database")
                        .value_name("PATH")
                        .short("e")
                        .long("database")
                        .help("Set the location of the database.")
                        .validator(path_check)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("volume")
                        .value_name("NUMBER")
                        .short("v")
                        .long("volume")
                        .help("Set the volume at startup.")
                        .validator(|v| {
                            if matches!(v.parse::<u64>().unwrap(), 0..=100) {
                                Ok(())
                            } else {
                                Err("The volume value must be between 0 and 100.".into())
                            }
                        })
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("debug")
                        .value_name("BOOL")
                        .short("d")
                        .long("debug")
                        .help("Debug or not.")
                        .takes_value(true)
                )
                .get_matches(),
        }
    }
}

impl Into<Config> for CLI<'_> {
    #[inline]
    fn into(self) -> Config {
        let args = &self.args;
        let theme = if let Some(t) = args.value_of("preset-theme") {
            match t {
                "Dark" => Some(Theme::dark()),
                "Light" => Some(Theme::light()),
                _ => None
            }
        } else {
            None
        };

        Config {
            overlay_background: args.value_of("overlay-background").map(|b| b.parse::<bool>().unwrap()),
            theme,
            lib_pos: args.value_of("INPUT").map(|l| l.to_owned()),
            db_pos: args.value_of("database").map(|d| d.to_owned()),
            volume: args.value_of("volume").map(|v| v.parse::<u64>().unwrap()),
            debug: args.value_of("debug").map(|b| b.parse::<bool>().unwrap()),
        }
    }
}
