use std::collections::HashMap;

use cosmic::iced::keyboard::Key;
use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::menu::key_bind::Modifier;

use crate::app::Action;

pub fn key_binds() -> HashMap<KeyBind, Action> {
    let mut key_binds = HashMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:ident) => {{
            key_binds.insert(
                KeyBind {
                    modifiers: vec![$(Modifier::$modifier),*],
                    key: $key,
                },
                Action::$action,
            );
        }};
    }

    bind!([Ctrl], Key::Character("n".into()), NewStudySet);
    bind!([Ctrl, Shift], Key::Character("b".into()), Backup);
    bind!([Ctrl, Shift], Key::Character("i".into()), Import);

    bind!([Ctrl, Shift], Key::Character("r".into()), RenameStudySet);
    bind!([Ctrl, Shift], Key::Character("d".into()), DeleteStudySet);

    bind!([Ctrl], Key::Character(",".into()), Settings);
    bind!([Ctrl], Key::Character("i".into()), About);

    key_binds
}
