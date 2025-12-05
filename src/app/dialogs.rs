// SPDX-License-Identifier: GPL-3.0

use std::{collections::VecDeque, sync::Arc};

use cosmic::{Element, Task, theme, widget};
use sqlx::{Pool, Sqlite};

use crate::{
    app::{
        Message, StudySet,
        core::{
            models::{
                flashcard::{Flashcard, FlashcardField},
                folder::Folder,
            },
            utils,
        },
    },
    fl,
};

/// Represents a [`DialogPage`] of the application
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    /// Dialog for creating a new [`StudySet`]
    NewStudySet(String),
    /// Dialog for renaming a [`StudySet`]
    RenameStudySet { to: String },
    /// Dialog for confirming the deletion of a [`StudySet`]
    DeleteStudySet,
    /// Dialog for confirming the deletion of a [`Folder`]
    DeleteFolder(i32),
    /// Dialog for creating a new [`Folder`]
    NewFolder(String),
    /// Dialog for confirming the deletion of a [`Flashcard`]
    DeleteFlashcard(Flashcard),
}

impl DialogPage {
    /// Handle the complete/ok action of each [`DialogPage`]
    pub fn complete(
        &self,
        database: &Arc<Pool<Sqlite>>,
        nav: &cosmic::widget::nav_bar::Model,
    ) -> Task<cosmic::Action<crate::app::Message>> {
        match &self {
            DialogPage::NewStudySet(studyset_name) => {
                if !studyset_name.is_empty() {
                    return Task::perform(
                        StudySet::add(Arc::clone(database), studyset_name.to_string()),
                        |result| match result {
                            Ok(_) => cosmic::action::app(Message::FetchStudySets),
                            Err(_) => cosmic::action::none(),
                        },
                    );
                }
                Task::none()
            }
            DialogPage::RenameStudySet { to: studyset_name } => {
                #[allow(clippy::collapsible_if)]
                if !studyset_name.is_empty() {
                    if let Some(set_id) = nav.active_data::<i32>() {
                        return Task::perform(
                            StudySet::edit(
                                Arc::clone(database),
                                StudySet {
                                    id: Some(*set_id),
                                    name: studyset_name.to_string(),
                                },
                            ),
                            move |result| match result {
                                Ok(_) => cosmic::action::app(Message::FetchStudySets),
                                Err(_) => cosmic::action::none(),
                            },
                        );
                    }
                }
                Task::none()
            }
            DialogPage::DeleteStudySet => {
                if let Some(set_id) = nav.active_data::<i32>() {
                    return Task::perform(
                        StudySet::delete(Arc::clone(database), *set_id),
                        move |result| match result {
                            Ok(_) => cosmic::action::app(Message::FetchStudySets),
                            Err(_) => cosmic::action::none(),
                        },
                    );
                }
                Task::none()
            }
            DialogPage::DeleteFolder(folder_id) => Task::perform(
                Folder::delete(Arc::clone(database), *folder_id),
                move |result| match result {
                    Ok(_) => cosmic::action::app(Message::Folders(
                        super::screen::folders::Message::LoadFolders,
                    )),
                    Err(_) => cosmic::action::none(),
                },
            ),
            DialogPage::NewFolder(folder_name) => {
                #[allow(clippy::collapsible_if)]
                if let Some(set_id) = nav.active_data::<i32>() {
                    if !folder_name.is_empty() {
                        let set_id_clone = *set_id;
                        return Task::perform(
                            Folder::add(
                                Arc::clone(database),
                                folder_name.to_string(),
                                set_id_clone,
                            ),
                            move |result| match result {
                                Ok(_folder_id) => {
                                    cosmic::action::app(Message::OpenFolders(set_id_clone))
                                }
                                Err(_) => cosmic::action::none(),
                            },
                        );
                    }
                }
                Task::none()
            }
            DialogPage::DeleteFlashcard(flashcard) => {
                let front_task = if let FlashcardField::Image { path, .. } = &flashcard.front {
                    // TODO: If this fails we can simply ignore it?
                    if !path.is_empty() && flashcard.id.is_some() {
                        Task::perform(utils::delete_image(path.to_string()), |_res| {
                            cosmic::action::none()
                        })
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                };

                let back_task = if let FlashcardField::Image { path, .. } = &flashcard.back {
                    if !path.is_empty() && flashcard.id.is_some() {
                        // TODO: If this fails we can simply ignore it?
                        Task::perform(utils::delete_image(path.to_string()), |_res| {
                            cosmic::action::none()
                        })
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                };

                Task::perform(
                    Flashcard::delete(Arc::clone(database), flashcard.id.unwrap_or_default()),
                    move |result| match result {
                        Ok(_) => cosmic::action::app(Message::Flashcards(
                            super::screen::flashcards::Message::LoadFlashcards,
                        )),
                        Err(_) => cosmic::action::none(),
                    },
                )
                .chain(front_task)
                .chain(back_task)
            }
        }
    }

    /// View of the [`DialogPage`]
    pub fn display(&self, dialog_state: &DialogState) -> Option<Element<Message>> {
        let spacing = theme::active().cosmic().spacing;

        let dialog = match &self {
            DialogPage::NewStudySet(studyset_name) => widget::dialog()
                .title(fl!("create-studyset"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press(Message::DialogAction(DialogAction::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("studyset-name")).into(),
                        widget::text_input("", studyset_name.as_str())
                            .id(dialog_state.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogAction(DialogAction::DialogUpdate(
                                    DialogPage::NewStudySet(name),
                                ))
                            })
                            .on_submit(|_x| Message::DialogAction(DialogAction::DialogComplete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::RenameStudySet { to: studyset_name } => widget::dialog()
                .title(fl!("rename-studyset"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press_maybe(Some(Message::DialogAction(DialogAction::DialogComplete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("studyset-name")).into(),
                        widget::text_input("", studyset_name.as_str())
                            .id(dialog_state.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogAction(DialogAction::DialogUpdate(
                                    DialogPage::RenameStudySet { to: name },
                                ))
                            })
                            .on_submit(|_x| Message::DialogAction(DialogAction::DialogComplete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::DeleteStudySet => widget::dialog()
                .title(fl!("delete-studyset"))
                .body(fl!("confirm-delete"))
                .primary_action(
                    widget::button::suggested(fl!("ok"))
                        .on_press_maybe(Some(Message::DialogAction(DialogAction::DialogComplete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                ),
            DialogPage::DeleteFolder(_folder_id) => widget::dialog()
                .title(fl!("delete-folder"))
                .body(fl!("confirm-delete"))
                .primary_action(
                    widget::button::suggested(fl!("ok"))
                        .on_press_maybe(Some(Message::DialogAction(DialogAction::DialogComplete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                ),
            DialogPage::NewFolder(folder_name) => widget::dialog()
                .title(fl!("create-folder"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press_maybe(Some(Message::DialogAction(DialogAction::DialogComplete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("folder-name")).into(),
                        widget::text_input("", folder_name.as_str())
                            .id(dialog_state.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogAction(DialogAction::DialogUpdate(
                                    DialogPage::NewFolder(name),
                                ))
                            })
                            .on_submit(|_x| Message::DialogAction(DialogAction::DialogComplete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::DeleteFlashcard(_flashcard) => widget::dialog()
                .title(fl!("delete-flashcard"))
                .body(fl!("confirm-delete"))
                .primary_action(
                    widget::button::suggested(fl!("ok"))
                        .on_press_maybe(Some(Message::DialogAction(DialogAction::DialogComplete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                ),
        };

        Some(dialog.into())
    }
}

/// Represents an Action related to a Dialog
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogAction {
    /// Asks to open the [`DialogPage`] for creating a new [`StudySet`]
    OpenNewStudySetDialog,
    /// Asks to open the [`DialogPage`] for renaming a [`StudySet`]
    OpenRenameStudySetDialog,
    /// Asks to open the [`DialogPage`] for confirming the deletion of a [`StudySet`]
    OpenDeleteStudySetDialog,
    /// Asks to open the [`DialogPage`] for creating a new [`Folder`]
    OpenCreateFolderDialog,
    /// Asks to open the [`DialogPage`] for confirming the deletion of a [`Folder`]
    OpenDeleteFolderDialog(i32),
    /// Asks to open the [`DialogPage`] for confirming the deletion of a [`Flashcard`]
    OpenDeleteFlashcardDialog(Flashcard),
    /// Action after user confirms/ok's/accepts the action of a Dialog
    DialogComplete,
    /// Action after user cancels the action of a Dialog
    DialogCancel,
    /// Updates the value of the given [`DialogPage`]
    DialogUpdate(DialogPage),
}

impl DialogAction {
    /// Executes the [`DialogAction`]
    pub fn execute(
        self,
        dialog_pages: &mut VecDeque<DialogPage>,
        dialog_state: &DialogState,
        database: &Arc<Pool<Sqlite>>,
        nav: &cosmic::widget::nav_bar::Model,
    ) -> Task<cosmic::Action<Message>> {
        match self {
            DialogAction::OpenNewStudySetDialog => {
                dialog_pages.push_back(DialogPage::NewStudySet(String::new()));
                widget::text_input::focus(dialog_state.dialog_text_input.clone())
            }
            DialogAction::OpenRenameStudySetDialog => {
                if let Some(set_name) = nav.text(nav.active()) {
                    dialog_pages.push_back(DialogPage::RenameStudySet {
                        to: set_name.to_string(),
                    });
                    return widget::text_input::focus(dialog_state.dialog_text_input.clone());
                }
                Task::none()
            }
            DialogAction::OpenDeleteStudySetDialog => {
                if nav.data::<i32>(nav.active()).is_some() {
                    dialog_pages.push_back(DialogPage::DeleteStudySet);
                }
                Task::none()
            }
            DialogAction::OpenCreateFolderDialog => {
                dialog_pages.push_back(DialogPage::NewFolder(String::new()));
                widget::text_input::focus(dialog_state.dialog_text_input.clone())
            }
            DialogAction::OpenDeleteFolderDialog(folder_id) => {
                dialog_pages.push_back(DialogPage::DeleteFolder(folder_id));
                Task::none()
            }
            DialogAction::OpenDeleteFlashcardDialog(flashcard) => {
                dialog_pages.push_back(DialogPage::DeleteFlashcard(flashcard));
                Task::none()
            }
            DialogAction::DialogComplete => {
                if let Some(dialog_page) = dialog_pages.pop_front() {
                    return dialog_page.complete(database, nav);
                }
                Task::none()
            }
            DialogAction::DialogCancel => {
                dialog_pages.pop_front();
                Task::none()
            }
            DialogAction::DialogUpdate(dialog_page) => {
                dialog_pages[0] = dialog_page;
                Task::none()
            }
        }
    }
}

/// State of all the dialog widgets of the app
pub struct DialogState {
    /// Input inside of the Dialog Pages of the Application
    pub dialog_text_input: widget::Id,
}

impl Default for DialogState {
    fn default() -> Self {
        Self {
            dialog_text_input: widget::Id::unique(),
        }
    }
}
