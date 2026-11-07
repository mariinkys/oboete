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
        Message::MenuAction(*self)
    }
}

//
// Responsive Menu Bar implementation based on cosmic-edit implementation (04/02/2026)
// Relevant links:
// https://github.com/pop-os/cosmic-edit/blob/master/src/menu.rs
// https://github.com/pop-os/cosmic-edit/blob/master/src/main.rs
//
// GNOME (Mutter) workaround:
// `responsive_menu_bar` opens its dropdowns as Wayland `xdg_popup` surfaces.
// On GNOME Wayland those popups are dismissed immediately, so the menus never
// open (see https://github.com/pop-os/libcosmic/issues/978). Outside COSMIC we
// fall back to the classic `menu::bar`, which draws the dropdowns as overlays
// inside the application window (they cannot extend past the window, and the
// bar does not collapse into an overflow button on narrow widths).
//

static MENU_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("responsive-menu"));

/// Whether the compositor is COSMIC, where popup-based menus work correctly.
static USE_POPUP_MENUS: LazyLock<bool> = LazyLock::new(|| {
    // Escape hatch for testing: COSMIC_APP_MENU_OVERLAY=1 forces overlay menus.
    if std::env::var_os("COSMIC_APP_MENU_OVERLAY").is_some() {
        return false;
    }
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|desktops| {
            desktops
                .split(':')
                .any(|d| d.eq_ignore_ascii_case("COSMIC"))
        })
        .unwrap_or(false)
});

/// Menu contents shared by both menu bar implementations.
fn menu_entries() -> Vec<(String, Vec<MenuItem<MenuAction, String>>)> {
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
    ]
}

pub fn menu_bar<'a>(core: &Core, key_binds: &HashMap<KeyBind, MenuAction>) -> Element<'a, Message> {
    if *USE_POPUP_MENUS {
        return responsive_menu_bar()
            .item_height(ItemHeight::Dynamic(40))
            .item_width(ItemWidth::Uniform(270))
            .spacing(4.0)
            .into_element(
                core,
                key_binds,
                MENU_ID.clone(),
                Message::Surface,
                menu_entries(),
            );
    }

    // GNOME / KDE / X11 / others: in-window overlay menus.
    let trees = menu_entries()
        .into_iter()
        .map(|(label, items)| {
            menu::Tree::with_children(
                Element::<'static, Message>::from(menu::root(label)),
                menu::items(key_binds, items),
            )
        })
        .collect();

    menu::bar(trees)
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(270))
        .spacing(4.0)
        .into()
}
