// SPDX-License-Identifier: GPL-3.0-only

use core::localization;

use app::Oboete;
use i18n_embed::DesktopLanguageRequester;
mod app;
mod core;
mod flashcards;
mod folders;
mod models;
mod utils;

/// The `cosmic::app::run()` function is the starting point of your application.
/// It takes two arguments:
/// - `settings` is a structure that contains everything relevant with your app's configuration, such as antialiasing, themes, icons, etc...
/// - `()` is the flags that your app needs to use before it starts.
///  If your app does not need any flags, you can pass in `()`.
fn main() -> cosmic::iced::Result {
    init_localizer();

    let (settings, flags) = core::settings::init();
    cosmic::app::run::<Oboete>(settings, flags)
}

fn init_localizer() {
    let localizer = localization::localizer();
    let requested_languages = DesktopLanguageRequester::requested_languages();

    if let Err(why) = localizer.select(&requested_languages) {
        panic!("can't load localizations: {}", why.to_string());
    }
}
