use std::sync::Mutex;

use cosmic::{
    app::Settings,
    iced::{Limits, Size},
};

use crate::app::Flags;

use super::{
    config::OboeteConfig,
    icon_cache::{IconCache, ICON_CACHE},
};

pub fn init() -> (Settings, Flags) {
    //set_logger();
    set_icon_cache();

    let settings = get_app_settings();
    let flags = get_flags();

    (settings, flags)
}

#[allow(dead_code)]
pub fn set_logger() {
    tracing_subscriber::fmt().json().init();
}

pub fn get_app_settings() -> Settings {
    let mut settings = Settings::default();

    settings = settings.size_limits(Limits::NONE.min_width(500.0).min_height(180.0));
    settings = settings.size(Size::new(1200.0, 800.0));
    settings = settings.debug(false);
    settings
}

pub fn get_flags() -> Flags {
    let (config_handler, config) = (OboeteConfig::config_handler(), OboeteConfig::config());

    let flags = Flags {
        config_handler,
        config,
    };
    flags
}

pub fn set_icon_cache() {
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
}
