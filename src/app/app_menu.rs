use crate::app::Message;
use cosmic::widget::menu;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    NewStudySet,
    Backup,
    Import,
    RenameStudySet,
    DeleteStudySet,
    About,
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
