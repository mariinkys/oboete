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

use crate::{
    fl, icons,
    oboete::{
        models::flashcard::Flashcard,
        utils::{export_flashcards, export_flashcards_anki, parse_ankifile, parse_import_content},
    },
};

pub struct FolderContent {
    current_folder_id: Option<i32>,
    // Flashcards inside the folder
    flashcards: Vec<Flashcard>,
    // State of the add/edit flashcard contextpage
    add_edit_flashcard: AddEditFlashcardState,

    // State of the folder options context page
    folder_options_state: FolderOptionsContextPageState,
}

struct AddEditFlashcardState {
    id: Option<i32>,
    front: String,
    back: String,
    status: i32,
}

impl AddEditFlashcardState {
    fn new() -> AddEditFlashcardState {
        AddEditFlashcardState {
            id: None,
            front: String::new(),
            back: String::new(),
            status: 0,
        }
    }
}

struct FolderOptionsContextPageState {
    pub between_terms: String,
    pub between_cards: String,
    pub import_content: String,
}

impl FolderOptionsContextPageState {
    fn new() -> FolderOptionsContextPageState {
        FolderOptionsContextPageState {
            between_terms: String::new(),
            between_cards: String::new(),
            import_content: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportOptions {
    Normal,
    Anki,
}

#[derive(Debug, Clone)]
pub enum Message {
    //FetchFolderFlashcards,
    EditFlashcard,
    AddFlashcard,
    DeleteFlashcard(Option<i32>),

    SetFlashcards(Vec<Flashcard>),
    EditedFlashcard,
    DeletedFlashcard,
    AddedNewFlashcard,

    // ADD/EDIT CONTEXT PAGE
    OpenAddEditContextPage(Flashcard),
    AddEditFlashcardFrontInput(String),
    AddEditFlashcardBackInput(String),
    RestartSingleFlashcardStatus(i32),

    // OPTIONS CONTEXT PAGE
    OpenFolderOptionsContextPage,
    FolderOptionsBetweenTermsInput(String),
    FolderOptionsBetweenCardsInput(String),
    FolderOptionsImportContentInput(String),
    ImportContent,
    ContentImported,
    OpenAnkiFileSelection,
    OpenAnkiFileResult(Vec<String>),
    LaunchUrl(String),
    RestartFolderFlashcardsStatus,
    OpenFolderExport(ExportOptions),
    OpenFolderExportResult(Vec<String>, ExportOptions),

    // Change to Study Page
    StudyFolder(i32),
}

pub enum FolderContentTask {
    FetchFolderFlashcards(i32),
    EditFlashcard(Flashcard),
    AddFlashcard(Flashcard),
    DeleteFlashcard(i32),

    OpenAddEditContextPage,
    CloseContextPage,
    RestartSingleFlashcardStatus(i32),

    OpenFolderOptionsContextPage,
    ImportContent(Vec<Flashcard>),
    OpenAnkiFileSelection,
    RestartFolderFlashcardsStatus(i32), // We pass the folder_id
    OpenFolderExport(ExportOptions),

    StudyFolder(i32),
}

impl FolderContent {
    pub fn init() -> Self {
        Self {
            current_folder_id: None,
            flashcards: Vec::new(),
            add_edit_flashcard: AddEditFlashcardState::new(),
            folder_options_state: FolderOptionsContextPageState::new(),
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

    /// Cleans the flashcards vec from the state
    pub fn clean_flashcards_vec(&mut self) {
        self.flashcards = Vec::new();
    }

    pub fn update(&mut self, message: Message) -> Vec<FolderContentTask> {
        let mut tasks = Vec::new();

        match message {
            //Asks to update the flashcard list of the current selected folder
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
            Message::AddEditFlashcardFrontInput(value) => {
                self.add_edit_flashcard.front = value;
            }

            // Updates the value of the flashcard back in the add/edit contextpage
            Message::AddEditFlashcardBackInput(value) => {
                self.add_edit_flashcard.back = value;
            }

            // Asks for a single flashcard status to be updated
            Message::RestartSingleFlashcardStatus(flashcard_id) => {
                tasks.push(FolderContentTask::RestartSingleFlashcardStatus(
                    flashcard_id,
                ));
            }

            // Asks to open the Folder Options Context Page
            Message::OpenFolderOptionsContextPage => {
                tasks.push(FolderContentTask::OpenFolderOptionsContextPage);
            }

            // Updates the value of the between_terms in the options contextpage
            Message::FolderOptionsBetweenTermsInput(value) => {
                self.folder_options_state.between_terms = value;
            }

            // Updates the value of the between_cards in the options contextpage
            Message::FolderOptionsBetweenCardsInput(value) => {
                self.folder_options_state.between_cards = value;
            }

            // Updates the value of the import content in the options contextpage
            Message::FolderOptionsImportContentInput(value) => {
                self.folder_options_state.import_content = value;
            }

            // Parses the content using the current state of the inputs and asks for it to be imported
            Message::ImportContent => {
                let content = parse_import_content(
                    &self.folder_options_state.between_cards,
                    &self.folder_options_state.between_terms,
                    &self.folder_options_state.import_content,
                );
                tasks.push(FolderContentTask::ImportContent(content))
            }

            // Callback after content has been successfully imported
            Message::ContentImported => {
                self.folder_options_state = FolderOptionsContextPageState::new();
                tasks.push(FolderContentTask::FetchFolderFlashcards(
                    self.current_folder_id.unwrap(),
                ));
                tasks.push(FolderContentTask::CloseContextPage);
            }

            // Asks for the app selection dialog to be opened
            Message::OpenAnkiFileSelection => {
                tasks.push(FolderContentTask::OpenAnkiFileSelection);
            }

            // Callback after anki file import dialog, tries to parse the content and asks to import it
            Message::OpenAnkiFileResult(result) => {
                if !result.is_empty() {
                    for path in result {
                        let flashcards = parse_ankifile(&path);
                        match flashcards {
                            Ok(flashcards) => {
                                tasks.push(FolderContentTask::ImportContent(flashcards))
                            }
                            Err(err) => eprintln!("{:?}", err),
                        }
                    }
                }
            }

            // Opens the given URL
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },

            // Asks for the current folder flashcard statuses to be restarted
            Message::RestartFolderFlashcardsStatus => {
                tasks.push(FolderContentTask::RestartFolderFlashcardsStatus(
                    self.current_folder_id.unwrap(),
                ));
            }

            // Asks to open the dialog to select the file export path
            Message::OpenFolderExport(options) => {
                if !self.flashcards.is_empty() {
                    tasks.push(FolderContentTask::OpenFolderExport(options));
                }
            }

            // Callback after the file export dialog, creates the correct file for the selected export
            Message::OpenFolderExportResult(save_result, options) => {
                match options {
                    ExportOptions::Normal => {
                        for path in save_result {
                            let _ = export_flashcards(&path, &self.flashcards);
                        }
                    }
                    ExportOptions::Anki => {
                        for path in save_result {
                            let _ = export_flashcards_anki(&path, &self.flashcards);
                        }
                    }
                };
            }

            // Asks for the study mode for a given folder (page change)
            Message::StudyFolder(folder_id) => {
                tasks.push(FolderContentTask::StudyFolder(folder_id));
            }
        }

        tasks
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        if !self.flashcards.is_empty() && self.current_folder_id.is_some() {
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

        let folder_options_button =
            widget::button::icon(icons::get_handle("emblem-system-symbolic", 18))
                .class(theme::Button::Standard)
                .on_press(Message::OpenFolderOptionsContextPage);

        widget::row::with_capacity(2)
            .align_y(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text::title3(fl!("flashcards")))
            .push(folder_options_button)
            .push(Space::new(Length::Fill, Length::Shrink))
            .push(study_button)
            .push(new_flashcard_button)
            .into()
    }

    /// The add/edit flashcard context page for this app.
    pub fn add_edit_flashcard_contextpage(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        // If we want a new flashcard the id will be none, if we want to edit it will be some
        let add_edit_button_label = if self.add_edit_flashcard.id.is_some() {
            fl!("edit")
        } else {
            fl!("create")
        };

        let add_edit_button = if !self.add_edit_flashcard.front.is_empty()
            && !self.add_edit_flashcard.back.is_empty()
        {
            // If we want a new flashcard the id will be none, if we want to edit it will be some
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

        let add_edit_section = widget::settings::view_column(vec![widget::settings::section()
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

        let reset_flashcard_status_column =
            widget::settings::view_column(vec![widget::settings::section()
                .title(fl!("reset-flashcard-title"))
                .into()]);

        let reset_flashcard_status_button = widget::button::text(fl!("reset-flashcard-button"))
            .on_press(Message::RestartSingleFlashcardStatus(
                self.add_edit_flashcard.id.unwrap_or_default(),
            ))
            .class(theme::Button::Destructive);

        match self.add_edit_flashcard.id.is_some() {
            // We are editing, we can reset the status
            true => widget::Column::new()
                .push(add_edit_section)
                .push(add_edit_button)
                .push(reset_flashcard_status_column)
                .push(reset_flashcard_status_button)
                .spacing(spacing.space_xs)
                .into(),

            // We are creating a new flashcard
            false => widget::Column::new()
                .push(add_edit_section)
                .push(add_edit_button)
                .spacing(spacing.space_xs)
                .into(),
        }
    }

    /// The options context page for each folder
    pub fn folder_options_contextpage(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        //  CUSTOM IMPORT
        let folder_import_btn = if !self.folder_options_state.import_content.is_empty()
            && !self.folder_options_state.between_cards.is_empty()
            && !self.folder_options_state.between_terms.is_empty()
        {
            widget::Row::new()
                .push(Space::new(Length::Fill, Length::Shrink))
                .push(
                    widget::button::text(fl!("import-button"))
                        .on_press(Message::ImportContent)
                        .class(theme::Button::Suggested),
                )
        } else {
            widget::Row::new()
                .push(Space::new(Length::Fill, Length::Shrink))
                .push(widget::button::text(fl!("import-button")).class(theme::Button::Suggested))
        };

        let folder_import_section =
            widget::settings::view_column(vec![widget::settings::section()
                .title(fl!("folder-import"))
                .add(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("import-between-term-title")).into(),
                        widget::text_input(
                            fl!("import-between-term-placeholder"),
                            &self.folder_options_state.between_terms,
                        )
                        .on_input(Message::FolderOptionsBetweenTermsInput)
                        .into(),
                        widget::text::body(fl!("import-between-cards-title")).into(),
                        widget::text_input(
                            fl!("import-between-cards-placeholder"),
                            &self.folder_options_state.between_cards,
                        )
                        .on_input(Message::FolderOptionsBetweenCardsInput)
                        .into(),
                        widget::text::body(fl!("import-content-title")).into(),
                        widget::text_input(
                            fl!("import-content-placeholder"),
                            &self.folder_options_state.import_content,
                        )
                        .on_input(Message::FolderOptionsImportContentInput)
                        .into(),
                    ])
                    .spacing(spacing.space_xxs),
                )
                .into()]);

        // ANKI IMPORT
        let anki_import_section = widget::settings::view_column(vec![widget::settings::section()
            .title(fl!("import-anki-title"))
            .add(
                widget::column::with_children(vec![widget::button::link(fl!(
                    "about-anki-importing"
                ))
                .on_press(Message::LaunchUrl(String::from(
                    "https://github.com/mariinkys/oboete/blob/main/info/ANKI_IMPORTING.md",
                )))
                .into()])
                .spacing(spacing.space_xxs),
            )
            .into()]);

        let anki_import_button = widget::Row::new()
            .push(Space::new(Length::Fill, Length::Shrink))
            .push(
                widget::button::text(fl!("import-button"))
                    .on_press(Message::OpenAnkiFileSelection)
                    .class(theme::Button::Suggested),
            );

        // RESET
        let reset_flashcards_status_section =
            widget::settings::view_column(vec![widget::settings::section()
                .title(fl!("reset-folder-flashcards-title"))
                .into()]);

        let reset_flashcards_status_button = if !self.flashcards.is_empty() {
            widget::button::text(fl!("reset-folder-flashcards-button"))
                .on_press(Message::RestartFolderFlashcardsStatus)
                .class(theme::Button::Destructive)
        } else {
            widget::button::text(fl!("reset-folder-flashcards-button"))
                .class(theme::Button::Destructive)
        };

        let reset_column = widget::Column::new()
            .push(reset_flashcards_status_section)
            .push(reset_flashcards_status_button)
            .spacing(spacing.space_xxxs);

        // EXPORT
        let export_flashcards_section =
            widget::settings::view_column(vec![widget::settings::section()
                .title(fl!("export-folder-flashcards-title"))
                .into()]);

        let export_flashcards_button = if !self.flashcards.is_empty() {
            widget::button::text(fl!("export-folder-flashcards-button"))
                .on_press(Message::OpenFolderExport(ExportOptions::Normal))
                .class(theme::Button::Suggested)
        } else {
            widget::button::text(fl!("export-folder-flashcards-button"))
                .class(theme::Button::Suggested)
        };

        let export_flashcards_anki_button = if !self.flashcards.is_empty() {
            widget::button::text(fl!("export-folder-flashcards-anki-button"))
                .on_press(Message::OpenFolderExport(ExportOptions::Anki))
                .class(theme::Button::Suggested)
        } else {
            widget::button::text(fl!("export-folder-flashcards-anki-button"))
                .class(theme::Button::Suggested)
        };

        let export_column = widget::Column::new()
            .push(export_flashcards_section)
            .push(export_flashcards_button)
            .push(export_flashcards_anki_button)
            .spacing(spacing.space_xxs);

        widget::Column::new()
            .push(folder_import_section)
            .push(folder_import_btn)
            .push(anki_import_section)
            .push(anki_import_button)
            .push(reset_column)
            .push(export_column)
            .spacing(spacing.space_xs)
            .into()
    }
}
