// SPDX-License-Identifier: GPL-3.0-only

use cosmic::prelude::*;
use cosmic::{
    iced::{
        Alignment, Length,
        alignment::{Horizontal, Vertical},
    },
    theme,
    widget::{self, Space},
};

use crate::{fl, icons, oboete::models::folder::Folder};

pub struct HomePage {
    current_set_id: Option<i32>,
    folders: Vec<Folder>,
    edit_folder: EditFolderState,
}

pub struct EditFolderState {
    id: Option<i32>,
    name: String,
    rtl_fix: bool,
}

impl EditFolderState {
    pub fn new() -> EditFolderState {
        EditFolderState {
            id: None,
            name: String::new(),
            rtl_fix: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    FetchSetFolders,
    EditFolder,
    DeleteFolder(Option<i32>),

    SetFolders(Vec<Folder>),
    EditedFolder,
    DeletedFolder,
    AddedNewFolder,

    OpenEditContextPage(Folder),
    EditFolderNameInput(String),
    ToggleRLTCheckboxValue,

    OpenCreateFolderDialog,

    OpenFolder(i32, bool),
}

pub enum HomePageTask {
    FetchSetFolders(i32),
    EditFolder(Folder),
    DeleteFolder(i32),

    OpenEditContextPage,
    CloseContextPage,

    OpenCreateFolderDialog,

    OpenFolder(i32, bool),
}

impl HomePage {
    pub fn init() -> Self {
        Self {
            current_set_id: None,
            folders: Vec::new(),
            edit_folder: EditFolderState::new(),
        }
    }

    /// Sets the value of the currently selected studyset
    pub fn set_current_studyset_id(&mut self, id: Option<i32>) {
        self.current_set_id = id;
    }

    /// Retrieves the currently selected studyset
    pub fn get_current_studyset_id(&self) -> Option<i32> {
        self.current_set_id
    }

    pub fn update(&mut self, message: Message) -> Vec<HomePageTask> {
        let mut tasks = Vec::new();

        match message {
            // Asks to update the folder list of the current selected set
            Message::FetchSetFolders => {
                if let Some(set_id) = self.current_set_id {
                    tasks.push(HomePageTask::FetchSetFolders(set_id));
                }
            }

            // Asks to edit the folder with the current state of the edit data
            Message::EditFolder => {
                tasks.push(HomePageTask::EditFolder(Folder {
                    id: self.edit_folder.id,
                    name: self.edit_folder.name.to_string(),
                    rtl_fix: self.edit_folder.rtl_fix,
                    flashcards: Vec::new(),
                }));
            }

            // Asks to delete the folder given the id (on delete button click...)
            Message::DeleteFolder(maybe_id) => {
                if let Some(id) = maybe_id {
                    tasks.push(HomePageTask::DeleteFolder(id));
                }
            }

            // Sets the given folders to the appstate
            Message::SetFolders(folders) => self.folders = folders,

            // After edit, reset the edit folder state, refresh the folder list and ask to close the edit context page
            Message::EditedFolder => {
                self.edit_folder = EditFolderState::new();
                if let Some(set_id) = self.current_set_id {
                    tasks.push(HomePageTask::FetchSetFolders(set_id));
                }
                tasks.push(HomePageTask::CloseContextPage);
            }

            // After delete, refresh the folder list
            Message::DeletedFolder => {
                if let Some(set_id) = self.current_set_id {
                    tasks.push(HomePageTask::FetchSetFolders(set_id));
                }
            }

            // Callback after a new folder has been added, refresh the folder list
            Message::AddedNewFolder => {
                if let Some(set_id) = self.current_set_id {
                    tasks.push(HomePageTask::FetchSetFolders(set_id));
                }
            }

            // Asks to open the edit context page and sets the editing folder state to the clicked folder
            Message::OpenEditContextPage(folder) => {
                self.edit_folder = EditFolderState {
                    id: folder.id,
                    name: folder.name,
                    rtl_fix: folder.rtl_fix,
                };
                tasks.push(HomePageTask::OpenEditContextPage);
            }

            // Updates the value of the folder name in the edit contextpage
            Message::EditFolderNameInput(value) => self.edit_folder.name = value,

            // Toggles the value of the RTL fix bool
            Message::ToggleRLTCheckboxValue => self.edit_folder.rtl_fix = !self.edit_folder.rtl_fix,

            // Asks for the create folder dialog to be open on main
            Message::OpenCreateFolderDialog => {
                tasks.push(HomePageTask::OpenCreateFolderDialog);
            }

            // Asks for the given folder to be opened (page change)
            Message::OpenFolder(folder_id, rtl_fix) => {
                tasks.push(HomePageTask::OpenFolder(folder_id, rtl_fix));
            }
        }

        tasks
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.current_set_id.is_some() {
            // Set is selected
            if !self.folders.is_empty() {
                // Set is selected and has folders
                let mut folders = widget::list::list_column().style(theme::Container::Card);

                // For each folder in the state
                for folder in &self.folders {
                    let edit_button = widget::button::icon(icons::get_handle("edit-symbolic", 18))
                        .class(theme::Button::Standard)
                        .width(Length::Shrink)
                        .on_press(Message::OpenEditContextPage(folder.clone()));

                    let delete_button =
                        widget::button::icon(icons::get_handle("user-trash-full-symbolic", 18))
                            .class(theme::Button::Destructive)
                            .on_press(Message::DeleteFolder(folder.id));

                    let open_button =
                        widget::button::icon(icons::get_handle("folder-open-symbolic", 18))
                            .class(theme::Button::Suggested)
                            .width(Length::Shrink)
                            .on_press(Message::OpenFolder(folder.id.unwrap(), folder.rtl_fix));

                    let folder_name = widget::text(folder.name.clone())
                        .align_y(Vertical::Center)
                        .align_x(Horizontal::Left)
                        .width(Length::Fill);

                    let row = widget::row::with_capacity(2)
                        .align_y(Alignment::Center)
                        .spacing(spacing.space_xxs)
                        .push(open_button)
                        .push(edit_button)
                        .push(folder_name)
                        .push(delete_button);

                    folders = folders.add(row);
                }

                widget::column::with_capacity(2)
                    .spacing(spacing.space_xxs)
                    .push(self.homepage_header_row())
                    .push(folders)
                    .apply(widget::container)
                    .height(Length::Shrink)
                    .apply(widget::scrollable)
                    .height(Length::Fill)
                    .into()
            } else {
                // Set is selected but has NO folders
                widget::column::with_capacity(2)
                    .spacing(spacing.space_xxs)
                    .push(self.homepage_header_row())
                    .push(
                        widget::Container::new(
                            widget::Text::new(fl!("empty-page")).size(spacing.space_xl),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(cosmic::iced::alignment::Horizontal::Center)
                        .align_y(cosmic::iced::alignment::Vertical::Center),
                    )
                    .height(Length::Fill)
                    .into()
            }
        } else {
            // No Set Selected
            let column = widget::Column::new()
                .push(
                    widget::Text::new(fl!("empty-page"))
                        .size(spacing.space_xl)
                        .align_x(Horizontal::Center)
                        .width(Length::Fill),
                )
                .push(
                    widget::Text::new(fl!("empty-page-noset"))
                        .size(spacing.space_l)
                        .align_x(Horizontal::Center)
                        .width(Length::Fill),
                )
                .width(Length::Fill);

            widget::Container::new(column)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .align_y(cosmic::iced::alignment::Vertical::Center)
                .into()
        }
    }

    fn homepage_header_row(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let new_folder_button = widget::button::icon(icons::get_handle("list-add-symbolic", 18))
            .class(theme::Button::Suggested)
            .on_press(Message::OpenCreateFolderDialog);

        widget::row::with_capacity(2)
            .align_y(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text::title3(fl!("folders")).width(Length::Fill))
            .push(new_folder_button)
            .into()
    }

    /// The edit folder context page for this app.
    pub fn edit_folder_contextpage(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let edit_button = if !self.edit_folder.name.is_empty() {
            widget::Row::new()
                .push(Space::new(Length::Fill, Length::Shrink))
                .push(
                    widget::button::text(fl!("edit"))
                        .on_press(Message::EditFolder)
                        .class(theme::Button::Suggested),
                )
        } else {
            widget::Row::new()
                .push(Space::new(Length::Fill, Length::Shrink))
                .push(widget::button::text(fl!("edit")).class(theme::Button::Suggested))
        };

        let settings = widget::settings::view_column(vec![
            widget::settings::section()
                .title(fl!("folder-details"))
                .add(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("folder-name")).into(),
                        widget::text_input(fl!("folder-name"), &self.edit_folder.name)
                            .on_input(Message::EditFolderNameInput)
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                )
                .add(
                    widget::column::with_children(vec![
                        //widget::text::body(fl!("folder-rtl-fix")).into(),
                        widget::checkbox(fl!("folder-rtl-fix"), self.edit_folder.rtl_fix)
                            .on_toggle(|_| Message::ToggleRLTCheckboxValue)
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                )
                .into(),
        ]);

        widget::Column::new()
            .push(settings)
            .push(edit_button)
            .spacing(spacing.space_xs)
            .into()
    }
}
