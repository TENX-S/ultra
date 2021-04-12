use crate::utils::{path_check, hex_to_rgb};
use crate::error::{anyhow, Result, InvalidVolume, NonexistentPresetTheme,};
use dirs_next::{audio_dir, config_dir, data_dir};
use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;
use std::fs::{self, File};
use std::collections::HashMap;
use tui::style::Color;

const BUILT_IN_THEMES: [&str; 2] = ["Dark", "Light"];

/// Configuration for ultra
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub overlay_background: Option<bool>,
    pub theme: Option<Theme>,
    pub lib_pos: Option<String>,
    pub db_pos: Option<String>,
    pub volume: Option<u64>,
    pub debug: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Theme {
    pub preset: Option<String>,
    background: Option<String>,
    focus: Option<String>,
    board_border: Option<String>,
    board_header: Option<String>,
    board_selected: Option<String>,
    board_unselected: Option<String>,
    search_border: Option<String>,
    query: Option<String>,
    timeline_border: Option<String>,
    spectrum_border: Option<String>,
}

impl Theme {

    #[inline]
    pub fn dark() -> Self {
        Theme {
            preset: Some("Dark".to_owned()),
            background: Some("#554236".to_owned()),       // https://nipponcolors.com/#kurotobi
            focus: Some("#f17c67".to_owned()),            // https://nipponcolors.com/#sangosyu
            board_border: Some("#BDC0BA".to_owned()),     // https://nipponcolors.com/#byakuroku
            board_header: Some("#58B2DC".to_owned()),     // https://nipponcolors.com/#sora
            board_selected: Some("#A8D8B9".to_owned()),   // https://nipponcolors.com/#byakuroku
            board_unselected: Some("#FEDFE1".to_owned()), // https://nipponcolors.com/#sakura
            search_border: Some("#554236".to_owned()),
            query: Some("#ffffff".to_owned()),
            timeline_border: Some("#BDC0BA".to_owned()),
            spectrum_border: Some("#BDC0BA".to_owned()),
        }
    }

    #[inline]
    pub fn light() -> Self {
        Theme {
            preset: Some("Light".to_owned()),
            background: Some("#fffffb".to_owned()),       // https://nipponcolors.com/#gofun
            focus: Some("#f17c67".to_owned()),
            board_border: Some("#4F4F48".to_owned()),     // https://nipponcolors.com/#dobunezumi
            board_header: Some("#BDC0BA".to_owned()),
            board_selected: Some("#854836".to_owned()),   // https://nipponcolors.com/#hiwada
            board_unselected: Some("#563F2E".to_owned()), // https://nipponcolors.com/#kogecha
            search_border: Some("#4F4F48".to_owned()),
            query: Some("#000000".to_owned()),
            timeline_border: Some("#4F4F48".to_owned()),
            spectrum_border: Some("#4F4F48".to_owned()),
        }
    }

    #[inline]
    pub fn colorscheme(&self) -> Result<HashMap<&'static str, Color>> {
        let mut map = HashMap::new();
        map.insert("background", hex_to_rgb(self.background.as_ref().unwrap())? );
        map.insert("focus", hex_to_rgb(self.focus.as_ref().unwrap())?);
        map.insert("board_border", hex_to_rgb(self.board_border.as_ref().unwrap())?);
        map.insert("board_header", hex_to_rgb(self.board_header.as_ref().unwrap())?);
        map.insert("board_selected", hex_to_rgb(self.board_selected.as_ref().unwrap())?);
        map.insert("board_unselected", hex_to_rgb(self.board_unselected.as_ref().unwrap())?);
        map.insert("search_border", hex_to_rgb(self.search_border.as_ref().unwrap())?);
        map.insert("query", hex_to_rgb(self.query.as_ref().unwrap())?);
        map.insert("timeline_border", hex_to_rgb(self.timeline_border.as_ref().unwrap())?);
        map.insert("spectrum_border", hex_to_rgb(self.spectrum_border.as_ref().unwrap())?);

        Ok(map)
    }

}

impl Default for Theme {
    #[inline]
    fn default() -> Self {
        Self::dark()
    }
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        let path_to_string = |p: PathBuf| p.to_str().unwrap().to_string();

        Config {
            overlay_background: Some(false),
            theme: Some(Default::default()),
            lib_pos: Some(path_to_string(audio_dir().unwrap())),
            db_pos: Some(path_to_string(data_dir().unwrap().join("Ultra"))),
            volume: Some(100),
            debug: Some(false)
        }
    }
}

macro_rules! fill_missing_val {
    ($config:expr, $default:expr, $($field:tt),* ) => {
        $(
        if $config.$field.is_none() {
            $config.$field = $default.$field.clone();
        }
        )*
    };
}

macro_rules! merge_cfg {
    ($sys:expr, $cli:expr, $($field:tt), *) => {{
        $(
        if $cli.$field.is_none() && $sys.$field.is_some() {
            $cli.$field = $sys.$field.clone();
        }
        )*
        $cli
    }};
}

impl Config {
    /// Load configuration from:
    ///
    /// |Platform | Value                                            | Example                                             |
    /// | ------- | ------------------------------------------------ | --------------------------------------------------- |
    /// | Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config/ultra.toml | /home/alice/.config/ultra.toml                      |
    /// | macOS   | `$HOME`/Library/Application Support/ultra.toml   | /Users/Alice/Library/Application Support/ultra.toml |
    /// | Windows | `{FOLDERID_RoamingAppData}\ultra.toml`           | C:\Users\Alice\AppData\Roaming                      |
    #[inline]
    pub fn load_sys() -> Result<Config> {
        let default_cfg = Config::default();
        let cfg_dir = config_dir().unwrap().join("Ultra");
        if !cfg_dir.exists() {
            fs::create_dir_all(&cfg_dir)?;
        }

        let cfg_pos = cfg_dir.join("ultra.toml");
        if !cfg_pos.exists() {
            File::create(&cfg_pos)?;
            fs::copy("ultra.toml", &cfg_pos)?;
            return Ok(default_cfg);
        }

        let mut cfg_file = File::open(cfg_pos)?;
        let mut config = String::new();
        cfg_file.read_to_string(&mut config)?;
        let mut cfg = toml::from_str::<Config>(&config)?;
        fill_missing_val!(
            cfg,
            default_cfg,
            overlay_background,
            theme,
            lib_pos,
            db_pos,
            volume,
            debug
        );
        Ok(cfg.check()?)
    }

    /// Merge config that from command line args with system configuration.
    #[inline]
    pub fn merge(self, mut cli: Self) -> Result<Self> {
        merge_cfg!(
        self,
        cli,
        overlay_background,
        theme,
        lib_pos,
        db_pos,
        volume,
        debug
        ).check()
    }

    #[inline]
    pub fn check(self) -> Result<Self> {
        path_check(self.lib_pos.as_ref().unwrap())?;
        path_check(self.db_pos.as_ref().unwrap())?;
        let cfg_theme = self.theme.as_ref().unwrap().preset.as_ref().unwrap().as_str();
        if !BUILT_IN_THEMES.contains(&cfg_theme) {
            return Err(anyhow!(NonexistentPresetTheme(cfg_theme.into())));
        }

        if !matches!(self.volume.unwrap(), 0..=100) {
            return Err(anyhow!(InvalidVolume));
        }

        Ok(self)
    }
}
