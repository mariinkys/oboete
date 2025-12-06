// SPDX-License-Identifier: GPL-3.0

use crate::app::Message;
use cosmic::widget::menu;

/// Represents a Action that executes after clicking on the application Menu
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    /// Create a new [`StudySet`]
    NewStudySet,
    /// Backup the entire application
    Backup,
    /// Import a backup of the entire application
    Import,
    /// Rename a [`StudySet`]
    RenameStudySet,
    /// Delete a [`StudySet`]
    DeleteStudySet,
    /// Open the About [`ContextPage`] of the application
    About,
    /// Open the Settings [`ContextPage`] of the application
    Settings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = crate::app::Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::MenuAction(MenuAction::About),
            MenuAction::NewStudySet => Message::MenuAction(MenuAction::NewStudySet),
            MenuAction::Backup => Message::MenuAction(MenuAction::Backup),
            MenuAction::Import => Message::MenuAction(MenuAction::Import),
            MenuAction::RenameStudySet => Message::MenuAction(MenuAction::RenameStudySet),
            MenuAction::DeleteStudySet => Message::MenuAction(MenuAction::DeleteStudySet),
            MenuAction::Settings => Message::MenuAction(MenuAction::Settings),
        }
    }
}
