// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use cosmic::cosmic_theme::Spacing;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::iced_widget::{column, row};
use cosmic::widget::{Row, Space, button, container, list, scrollable, settings, text, text_input};
use cosmic::{Element, Task, theme};
use sqlx::{Pool, Sqlite};

use crate::app::context_page::ContextPage;
use crate::app::core::models::folder::Folder;
use crate::{fl, icons};

pub struct FoldersScreen {
    state: State,
}

enum State {
    Loading,
    NoStudySet,
    Ready {
        current_set_id: Option<i32>,
        edit_folder: Folder,
        folders: Vec<Folder>,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateCurrentSetId(i32),
    LoadFolders,
    FoldersLoaded(Result<Vec<Folder>, anywho::Error>),

    OpenCreateFolderDialog,
    OpenContextPage(ContextPage, Folder),

    EditFolder,
    EditFolderInput(String),

    DeleteFolder(i32),

    OpenFolder(i32),
}

pub enum Action {
    None,
    Run(Task<Message>),

    OpenCreateFolderDialog,
    OpenDeleteFolderDialog(i32),
    OpenContextPage(ContextPage),

    OpenFolder(i32),
}

impl FoldersScreen {
    pub fn new(database: &Arc<Pool<Sqlite>>, studyset_id: Option<i32>) -> (Self, Task<Message>) {
        if let Some(set_id) = studyset_id {
            (
                Self {
                    state: State::Loading,
                },
                Task::perform(
                    Folder::get_all(Arc::clone(database), set_id),
                    Message::FoldersLoaded,
                )
                .chain(Task::done(Message::UpdateCurrentSetId(set_id))),
            )
        } else {
            (
                Self {
                    state: State::NoStudySet,
                },
                Task::none(),
            )
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => container(text("Loading...")).center(Length::Fill).into(),
            State::NoStudySet => container(text("Nothing to see here..."))
                .center(Length::Fill)
                .into(),
            State::Ready { folders, .. } => {
                let spacing = theme::active().cosmic().spacing;

                let header = header_view(spacing);
                let content = folders_view(&spacing, folders);

                container(
                    column![header, content]
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .spacing(spacing.space_s),
                )
                .center(Length::Fill)
                .into()
            }
        }
    }

    pub fn update(&mut self, message: Message, database: &Arc<Pool<Sqlite>>) -> Action {
        match message {
            Message::UpdateCurrentSetId(set_id) => {
                let State::Ready { current_set_id, .. } = &mut self.state else {
                    return Action::None;
                };

                *current_set_id = Some(set_id);
                Action::None
            }
            Message::LoadFolders => {
                let State::Ready { current_set_id, .. } = &self.state else {
                    return Action::None;
                };

                if let Some(current_set_id) = *current_set_id {
                    Action::Run(
                        Task::perform(
                            Folder::get_all(Arc::clone(database), current_set_id),
                            Message::FoldersLoaded,
                        )
                        .chain(Task::done(Message::UpdateCurrentSetId(current_set_id))),
                    )
                } else {
                    Action::None
                }
            }
            Message::FoldersLoaded(res) => {
                match res {
                    Ok(folders) => {
                        if let State::Ready { edit_folder, .. } = &self.state {
                            self.state = State::Ready {
                                current_set_id: None,
                                folders,
                                edit_folder: edit_folder.clone(),
                            }
                        } else {
                            self.state = State::Ready {
                                current_set_id: None,
                                folders,
                                edit_folder: Folder::default(),
                            }
                        }
                    }
                    Err(e) => {
                        // TODO: Error Handling
                        eprintln!("{}", e);
                    }
                }
                Action::None
            }

            Message::OpenCreateFolderDialog => Action::OpenCreateFolderDialog,

            Message::OpenContextPage(context_page, folder) => {
                let State::Ready { edit_folder, .. } = &mut self.state else {
                    return Action::None;
                };

                *edit_folder = folder;

                Action::OpenContextPage(context_page)
            }

            Message::EditFolder => {
                let State::Ready { edit_folder, .. } = &self.state else {
                    return Action::None;
                };

                Action::Run(Task::perform(
                    Folder::edit(Arc::clone(database), edit_folder.clone()),
                    |res| match res {
                        Ok(_) => Message::LoadFolders,
                        Err(e) => {
                            // TODO: Error Handling
                            eprintln!("{}", e);
                            Message::LoadFolders
                        }
                    },
                ))
            }
            Message::EditFolderInput(value) => {
                let State::Ready { edit_folder, .. } = &mut self.state else {
                    return Action::None;
                };

                edit_folder.name = value;

                Action::None
            }

            Message::DeleteFolder(folder_id) => Action::OpenDeleteFolderDialog(folder_id),

            Message::OpenFolder(folder_id) => Action::OpenFolder(folder_id),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    //
    // CONTEXT PAGES
    //

    pub fn folder_settings<'a>(&'a self, spacing: Spacing) -> Element<'a, Message> {
        let State::Ready { edit_folder, .. } = &self.state else {
            return text("Error").into(); // It's theoretically impossible to be here.
        };

        let edit_button = Row::new()
            .push(Space::new(Length::Fill, Length::Shrink))
            .push(
                button::text(fl!("edit"))
                    .on_press_maybe((!edit_folder.name.is_empty()).then_some(Message::EditFolder))
                    .class(theme::Button::Suggested),
            );

        let settings = settings::view_column(vec![
            settings::section()
                .title(fl!("folder-details"))
                .add(
                    cosmic::widget::column::with_children(vec![
                        text::body(fl!("folder-name")).into(),
                        text_input(fl!("folder-name"), &edit_folder.name)
                            .on_input(Message::EditFolderInput)
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                )
                .into(),
        ]);

        column![settings, edit_button]
            .spacing(spacing.space_xs)
            .into()
    }
}

//
// VIEWS
//

fn header_view<'a>(spacing: Spacing) -> Element<'a, Message> {
    let new_folder_button = button::icon(icons::get_handle("list-add-symbolic", 18))
        .class(theme::Button::Suggested)
        .on_press(Message::OpenCreateFolderDialog);

    cosmic::widget::row::with_capacity(2)
        .align_y(Alignment::Center)
        .spacing(spacing.space_s)
        .padding([spacing.space_none, spacing.space_xxs])
        .push(text::title3(fl!("folders")).width(Length::Fill))
        .push(new_folder_button)
        .into()
}

fn folders_view<'a>(spacing: &Spacing, folders: &'a [Folder]) -> Element<'a, Message> {
    let content: Element<'a, Message> = if folders.is_empty() {
        text("Create some folders to get started...").into()
    } else {
        let mut folders_list = list::list_column().style(theme::Container::Card);

        for folder in folders {
            folders_list = folders_list.add(
                row![
                    row![
                        button::icon(icons::get_handle("folder-open-symbolic", 18))
                            .class(theme::Button::Suggested)
                            .width(Length::Shrink)
                            .on_press(Message::OpenFolder(folder.id.unwrap_or_default())),
                        button::icon(icons::get_handle("edit-symbolic", 18))
                            .class(theme::Button::Standard)
                            .width(Length::Shrink)
                            .on_press(Message::OpenContextPage(
                                ContextPage::FolderSettings,
                                folder.clone()
                            ))
                    ]
                    .spacing(spacing.space_xxs),
                    text(folder.name.clone())
                        .align_y(Vertical::Center)
                        .align_x(Horizontal::Left)
                        .width(Length::Fill),
                    button::icon(icons::get_handle("user-trash-full-symbolic", 18))
                        .class(theme::Button::Destructive)
                        .on_press(Message::DeleteFolder(folder.id.unwrap_or_default()))
                ]
                .width(Length::Fill)
                .align_y(Alignment::Center)
                .spacing(spacing.space_s),
            );
        }

        folders_list.into()
    };

    scrollable(
        container(content)
            .align_x(Horizontal::Center)
            .width(Length::Fill),
    )
    .into()
}
