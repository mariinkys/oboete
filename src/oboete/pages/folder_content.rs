// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length,
    },
    theme,
    widget::{self, Space},
    Apply, Element,
};

use crate::{fl, icons, oboete::models::flashcard::Flashcard};

pub struct FolderContent {
    current_folder_id: Option<i32>,
    flashcards: Vec<Flashcard>,
    add_edit_flashcard: AddEditFlashcardState,
}

pub struct AddEditFlashcardState {
    id: Option<i32>,
    front: String,
    back: String,
    status: i32,
}

impl AddEditFlashcardState {
    pub fn new() -> AddEditFlashcardState {
        AddEditFlashcardState {
            id: None,
            front: String::new(),
            back: String::new(),
            status: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    // FetchFolderFlashcards,
    EditFlashcard,
    AddFlashcard,
    DeleteFlashcard(Option<i32>),

    SetFlashcards(Vec<Flashcard>),
    EditedFlashcard,
    DeletedFlashcard,
    AddedNewFlashcard,

    OpenAddEditContextPage(Flashcard),
    AddEditFlashcardFrontInput(String),
    AddEditFlashcardBackInput(String),

    StudyFolder(i32),
}

pub enum FolderContentTask {
    FetchFolderFlashcards(i32),
    EditFlashcard(Flashcard),
    AddFlashcard(Flashcard),
    DeleteFlashcard(i32),

    OpenAddEditContextPage,
    CloseContextPage,

    StudyFolder(i32),
}

impl FolderContent {
    pub fn init() -> Self {
        Self {
            current_folder_id: None,
            flashcards: Vec::new(),
            add_edit_flashcard: AddEditFlashcardState::new(),
        }
    }

    /// Sets the value of the currently selected folder
    pub fn set_current_folder_id(&mut self, id: Option<i32>) {
        self.current_folder_id = id;
    }

    /// Retrieves the currently selected folder_id
    pub fn get_current_folder_id(&self) -> Option<i32> {
        self.current_folder_id
    }

    pub fn update(&mut self, message: Message) -> Vec<FolderContentTask> {
        let mut tasks = Vec::new();

        match message {
            // Asks to update the flashcard list of the current selected folder
            // Message::FetchFolderFlashcards => {
            //     if let Some(folder_id) = self.current_folder_id {
            //         tasks.push(FolderContentTask::FetchFolderFlashcards(folder_id));
            //     }
            // }

            // Asks to edit the flashcard with the current state of the add/edit data
            Message::EditFlashcard => {
                tasks.push(FolderContentTask::EditFlashcard(Flashcard {
                    id: self.add_edit_flashcard.id,
                    front: self.add_edit_flashcard.front.to_string(),
                    back: self.add_edit_flashcard.back.to_string(),
                    status: self.add_edit_flashcard.status,
                }));
            }

            // Asks to add the flashcard with the current state of the add/edit data
            Message::AddFlashcard => {
                tasks.push(FolderContentTask::AddFlashcard(Flashcard {
                    id: None,
                    front: self.add_edit_flashcard.front.to_string(),
                    back: self.add_edit_flashcard.back.to_string(),
                    status: 0,
                }));
            }

            // Asks to delete the flashcard given the id (on delete button click...)
            Message::DeleteFlashcard(maybe_id) => {
                if let Some(id) = maybe_id {
                    tasks.push(FolderContentTask::DeleteFlashcard(id));
                }
            }

            // Sets the given flashcards to the appstate
            Message::SetFlashcards(flashcards) => {
                self.flashcards = flashcards;
            }

            // After edit, reset the edit flashcard state, refresh the flashcard list and ask to close the edit context page
            Message::EditedFlashcard => {
                self.add_edit_flashcard = AddEditFlashcardState::new();
                if let Some(folder_id) = self.current_folder_id {
                    tasks.push(FolderContentTask::FetchFolderFlashcards(folder_id));
                }
                tasks.push(FolderContentTask::CloseContextPage);
            }

            // After delete, refresh the flashcard list
            Message::DeletedFlashcard => {
                if let Some(folder_id) = self.current_folder_id {
                    tasks.push(FolderContentTask::FetchFolderFlashcards(folder_id));
                }
            }

            // Callback after a new flashcard has been added, refresh the flashcard list
            Message::AddedNewFlashcard => {
                if let Some(folder_id) = self.current_folder_id {
                    tasks.push(FolderContentTask::FetchFolderFlashcards(folder_id));
                }
                tasks.push(FolderContentTask::CloseContextPage);
            }

            // Asks to open the edit context page and sets the editing flashcard state to the clicked flashcard
            Message::OpenAddEditContextPage(flashcard) => {
                self.add_edit_flashcard = AddEditFlashcardState {
                    id: flashcard.id,
                    front: flashcard.front,
                    back: flashcard.back,
                    status: flashcard.status,
                };
                tasks.push(FolderContentTask::OpenAddEditContextPage);
            }

            // Updates the value of the flashcard front in the add/edit contextpage
            Message::AddEditFlashcardFrontInput(value) => self.add_edit_flashcard.front = value,

            // Updates the value of the flashcard back in the add/edit contextpage
            Message::AddEditFlashcardBackInput(value) => self.add_edit_flashcard.back = value,

            // Asks for the study mode for a given folder (page change)
            Message::StudyFolder(folder_id) => {
                tasks.push(FolderContentTask::StudyFolder(folder_id));
            }
        }

        tasks
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;
        assert!(self.current_folder_id.is_some());

        if !self.flashcards.is_empty() {
            // folder is selected and has flashcards
            let mut flashcards = widget::list::list_column().style(theme::Container::Card);

            // For each folder in the state
            for flashcard in &self.flashcards {
                // If we want to edit a flashcard we pass an id: Some(id)
                let edit_button = widget::button::icon(icons::get_handle("edit-symbolic", 18))
                    .class(theme::Button::Standard)
                    .width(Length::Shrink)
                    .on_press(Message::OpenAddEditContextPage(flashcard.clone()));

                let delete_button =
                    widget::button::icon(icons::get_handle("user-trash-full-symbolic", 18))
                        .class(theme::Button::Destructive)
                        .on_press(Message::DeleteFlashcard(flashcard.id));

                let flashcard_front = widget::text(flashcard.front.clone())
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Left)
                    .width(Length::Fill);

                //TODO: Custom Button to make it look like a badge
                let badge = widget::text(match flashcard.status {
                    1 => format!("{}     ", fl!("bad-status")), // High chance (status = 1 = flashcard Bad)
                    2 => format!("{}     ", fl!("ok-status")), // Medium chance (status = 2 = flashcard Ok)
                    3 => format!("{}     ", fl!("good-status")), // Low chance (status = 3 = flashcard Good)
                    _ => String::new(), // Default chance for other statuses
                })
                .align_y(Vertical::Center)
                .align_x(Horizontal::Right)
                .width(Length::Shrink);

                let row = widget::row::with_capacity(2)
                    .align_y(Alignment::Center)
                    .spacing(spacing.space_xxs)
                    .push(edit_button)
                    .push(flashcard_front)
                    .push(badge)
                    .push(delete_button);

                flashcards = flashcards.add(row);
            }

            widget::column::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(self.folder_content_header_row())
                .push(flashcards)
                .apply(widget::container)
                .height(Length::Shrink)
                .apply(widget::scrollable)
                .height(Length::Fill)
                .into()
        } else {
            // folder is selected but has NO flashcards
            widget::column::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(self.folder_content_header_row())
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
    }

    fn folder_content_header_row(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        // If we want a new flashcard we pass an id: None
        let new_flashcard_button = widget::button::icon(icons::get_handle("list-add-symbolic", 18))
            .class(theme::Button::Suggested)
            .on_press(Message::OpenAddEditContextPage(Flashcard {
                id: None,
                front: String::new(),
                back: String::new(),
                status: 0,
            }));

        let study_button = if !self.flashcards.is_empty() {
            widget::button::text("Study")
                .class(theme::Button::Suggested)
                .on_press(Message::StudyFolder(self.current_folder_id.unwrap()))
        } else {
            widget::button::text("Study").class(theme::Button::Suggested)
        };

        widget::row::with_capacity(2)
            .align_y(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text::title3(fl!("flashcards")).width(Length::Fill))
            .push(study_button)
            .push(new_flashcard_button)
            .into()
    }

    /// The add/edit flashcard context page for this app.
    pub fn add_edit_flashcard_contextpage(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let add_edit_button_label = if self.add_edit_flashcard.id.is_some() {
            fl!("edit")
        } else {
            fl!("create")
        };

        let add_edit_button = if !self.add_edit_flashcard.front.is_empty()
            && !self.add_edit_flashcard.back.is_empty()
        {
            let btn_on_press = if self.add_edit_flashcard.id.is_some() {
                Message::EditFlashcard
            } else {
                Message::AddFlashcard
            };

            widget::Row::new()
                .push(Space::new(Length::Fill, Length::Shrink))
                .push(
                    widget::button::text(add_edit_button_label)
                        .on_press(btn_on_press)
                        .class(theme::Button::Suggested),
                )
        } else {
            widget::Row::new()
                .push(Space::new(Length::Fill, Length::Shrink))
                .push(widget::button::text(add_edit_button_label).class(theme::Button::Suggested))
        };

        let settings = widget::settings::view_column(vec![widget::settings::section()
            .title(fl!("flashcard-options"))
            .add(
                widget::column::with_children(vec![
                    widget::text::body(fl!("flashcard-front-title")).into(),
                    widget::text_input(
                        fl!("flashcard-front-placeholder"),
                        &self.add_edit_flashcard.front,
                    )
                    .on_input(Message::AddEditFlashcardFrontInput)
                    .into(),
                    widget::text::body(fl!("flashcard-back-title")).into(),
                    widget::text_input(
                        fl!("flashcard-back-placeholder"),
                        &self.add_edit_flashcard.back,
                    )
                    .on_input(Message::AddEditFlashcardBackInput)
                    .into(),
                ])
                .spacing(spacing.space_xxs),
            )
            .into()]);

        widget::Column::new()
            .push(settings)
            .push(add_edit_button)
            .spacing(spacing.space_xs)
            .into()
    }
}
