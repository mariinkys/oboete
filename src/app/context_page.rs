// SPDX-License-Identifier: GPL-3.0

use cosmic::{app::context_drawer, theme};

use crate::{
    app::{AppModel, Message},
    fl,
};

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
    FolderSettings,
    AddEditFlashcard,
    FolderContentOptions,
}

impl ContextPage {
    pub fn display<'a>(
        &self,
        app_model: &'a AppModel,
    ) -> Option<context_drawer::ContextDrawer<'a, Message>> {
        let spacing = theme::active().cosmic().spacing;

        Some(match &self {
            ContextPage::About => context_drawer::about(
                &app_model.about,
                |s| Message::LaunchUrl(s.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
            ContextPage::Settings => context_drawer::context_drawer(
                app_model.settings(),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title(fl!("folder-details")),
            ContextPage::FolderSettings => {
                let crate::app::State::Ready { screen, .. } = &app_model.state else {
                    return None;
                };

                let crate::app::Screen::Folders(folders_screen) = &screen else {
                    return None;
                };

                context_drawer::context_drawer(
                    folders_screen
                        .folder_settings(spacing)
                        .map(Message::Folders),
                    Message::ToggleContextPage(ContextPage::FolderSettings),
                )
                .title(fl!("folder-details"))
            }
            ContextPage::AddEditFlashcard => {
                let crate::app::State::Ready { screen, .. } = &app_model.state else {
                    return None;
                };

                let crate::app::Screen::Flashcards(flashcards_screen) = &screen else {
                    return None;
                };

                context_drawer::context_drawer(
                    flashcards_screen
                        .add_edit_contextpage(spacing)
                        .map(Message::Flashcards),
                    Message::ToggleContextPage(ContextPage::AddEditFlashcard),
                )
                .title(fl!("flashcard-options"))
            }
            ContextPage::FolderContentOptions => {
                let crate::app::State::Ready { screen, .. } = &app_model.state else {
                    return None;
                };

                let crate::app::Screen::Flashcards(flashcards_screen) = &screen else {
                    return None;
                };

                context_drawer::context_drawer(
                    flashcards_screen
                        .options_contextpage(spacing)
                        .map(Message::Flashcards),
                    Message::ToggleContextPage(ContextPage::FolderContentOptions),
                )
                .title(fl!("flashcard-options"))
            }
        })
    }
}
