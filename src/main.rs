// SPDX-License-Identifier: GPL-3.0-only
#![allow(clippy::collapsible_if)]
#![allow(mismatched_lifetime_syntaxes)]

use crate::flags::flags;
use std::sync::Mutex;

mod app;
mod config;
mod flags;
mod i18n;
mod icons;
mod key_binds;
mod oboete;

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default().size(cosmic::iced::Size::new(1200.0, 800.0));

    // Init the icon cache
    icons::ICON_CACHE.get_or_init(|| Mutex::new(icons::IconCache::new()));

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::Oboete>(settings, flags())
}
