// SPDX-License-Identifier: GPL-3.0

use crate::{app::Message, fl};
use cosmic::widget::menu::Item as MenuItem;
use cosmic::{
    Core, Element,
    widget::{
        menu::{self, ItemHeight, ItemWidth, KeyBind},
        responsive_menu_bar,
    },
};
use std::{collections::HashMap, sync::LazyLock};

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

//
// Responsive Menu Bar implementation based on cosmic-edit implementation (04/02/2026)
// Relevant links:
// https://github.com/pop-os/cosmic-edit/blob/master/src/menu.rs
// https://github.com/pop-os/cosmic-edit/blob/master/src/main.rs
//

static MENU_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("responsive-menu"));

pub fn menu_bar<'a>(core: &Core, key_binds: &HashMap<KeyBind, MenuAction>) -> Element<'a, Message> {
    responsive_menu_bar()
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(270))
        .spacing(4.0)
        .into_element(
            core,
            key_binds,
            MENU_ID.clone(),
            Message::Surface, // You may need to add this message variant
            vec![
                (
                    fl!("file"),
                    vec![
                        MenuItem::Button(fl!("new-studyset"), None, MenuAction::NewStudySet),
                        MenuItem::Button(fl!("backup"), None, MenuAction::Backup),
                        MenuItem::Button(fl!("import"), None, MenuAction::Import),
                    ],
                ),
                (
                    fl!("edit"),
                    vec![
                        MenuItem::Button(fl!("rename-studyset"), None, MenuAction::RenameStudySet),
                        MenuItem::Button(fl!("delete-studyset"), None, MenuAction::DeleteStudySet),
                    ],
                ),
                (
                    fl!("view"),
                    vec![
                        MenuItem::Button(fl!("about"), None, MenuAction::About),
                        MenuItem::Button(fl!("settings"), None, MenuAction::Settings),
                    ],
                ),
            ],
        )
}
