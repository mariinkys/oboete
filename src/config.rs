// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    cosmic_config::{self, Config, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

const CONFIG_VERSION: u64 = 1;
const APP_ID: &str = "dev.mariinkys.Oboete";

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
pub struct OboeteConfig {
    pub app_theme: AppTheme,
}

impl OboeteConfig {
    pub fn config_handler() -> Option<Config> {
        Config::new(APP_ID, CONFIG_VERSION).ok()
    }

    pub fn config() -> OboeteConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                OboeteConfig::get_entry(&config_handler).unwrap_or_else(|(error, config)| {
                    eprintln!("Error whilst loading config: {error:#?}");
                    config
                })
            }
            None => OboeteConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AppTheme {
    Dark,
    Light,
    #[default]
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => theme::Theme::dark(),
            Self::Light => theme::Theme::light(),
            Self::System => theme::system_preference(),
        }
    }
}
