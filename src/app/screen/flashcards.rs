// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, ContentFit, Length, Subscription};
use cosmic::iced_widget::{column, pick_list, row, stack};
use cosmic::widget::{
    Row, Space, button, container, image, list, scrollable, settings, text, text_input,
};
use cosmic::{Element, Task, theme};
use percent_encoding::percent_decode;
use sqlx::{Pool, Sqlite};

use crate::app::context_page::ContextPage;
use crate::app::core::models::flashcard::{Flashcard, FlashcardField};
use crate::app::core::utils::{self, OboeteToast};
use crate::app::widgets::pill::pill;
use crate::{fl, icons};

/// Screen [`State`] holder
pub struct FlashcardsScreen {
    state: State,
}

/// The different states this screen can be in
enum State {
    Loading,
    Ready {
        current_folder_id: Option<i32>,
        add_edit_flashcard: Box<Flashcard>,
        flashcards: Vec<Flashcard>,
        options: FolderOptions,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Does nothing
    None,
    /// Show the user a toast
    AddToast(OboeteToast),
    /// Opens the given url on the browser
    LaunchUrl(String),
    /// Update the current folder id, needed for some database operations
    UpdateCurrentFolderId(i32),
    /// Load the flashcards into state
    LoadFlashcards,
    /// Callback after asking to load the flashcards into state
    FlashcardsLoaded(Result<Vec<Flashcard>, anywho::Error>),

    /// Ask to open the given [`ContextPage`] for the given [`Flashcard`] (if necessary)
    OpenContextPage(ContextPage, Option<Flashcard>),

    /// Ask to edit a [`Flashcard`] in the database
    EditFlashcard,
    /// Ask to add a [`Flashcard`] in the database
    AddFlashcard,
    /// Callback after user input that's either adding or editing a [`Flashcard`]
    AddEditFlashcardInput(AddEditFlashcardInput),
    /// Resets the given [`Flashcard`] status, it also resets it's FSRS Data
    ResetFlashcardStatus(i32),

    /// Delete the given [`Flashcard`] from the database, also deletes any image the [`Flashcard`] may have
    DeleteFlashcard(Flashcard),

    /// User input on the folder options [`ContextPage`]
    FolderOptionsInput(FolderOptionsInput),

    /// Ask to open the study page of the current folder
    Study,
}

/// Represents the different inputs the user can perfrom on the flashcard upsert [`ContextPage`]
#[derive(Debug, Clone)]
pub enum AddEditFlashcardInput {
    FrontFieldTypeChanged(FlashcardField),
    FrontTextInput(String),
    FrontSelectImage,
    FrontImageSelected(String),
    DeleteFrontSelectedImage,
    FrontAltTextInput(String),

    BackFieldTypeChanged(FlashcardField),
    BackTextInput(String),
    BackSelectImage,
    BackImageSelected(String),
    DeleteBackSelectedImage,
    BackAltTextInput(String),
}

/// Represents the different inputs the user can perfrom on the folder options [`ContextPage`]
#[derive(Debug, Clone)]
pub enum FolderOptionsInput {
    ImportContentInput(String),
    BetweenCardsInput(String),
    BetweenTermsInput(String),
    CustomImport,

    AnkiImport,
    CompleteAnkiImport(String),

    ResetAllStatus,

    Export,
    CompleteExport(String),
    ExportAnki,
    CompleteAnkiExport(String),
}

/// Allows us to talk with the parent screen
pub enum Action {
    None,
    Run(Task<Message>),
    AddToast(OboeteToast),

    OpenDeleteFlashcardDialog(Flashcard),
    OpenContextPage(ContextPage),

    StudyFolder(i32),
}

/// State holder for the folder options [`ContextPage`]
#[derive(Debug, Default)]
struct FolderOptions {
    import_content: String,
    between_cards: String,
    between_terms: String,
}

impl FolderOptions {
    /// Returns true if the folder options can be submitted to perform an action on the database
    pub fn is_valid(&self) -> bool {
        !self.import_content.is_empty()
            && !self.between_cards.is_empty()
            && !self.between_terms.is_empty()
    }
}

impl FlashcardsScreen {
    /// Init the screen
    pub fn new(database: &Arc<Pool<Sqlite>>, folder_id: i32) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(
                Flashcard::get_all(Arc::clone(database), folder_id),
                Message::FlashcardsLoaded,
            )
            .chain(Task::done(Message::UpdateCurrentFolderId(folder_id))),
        )
    }

    /// View of the screen
    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => container(text(fl!("loading"))).center(Length::Fill).into(),
            State::Ready { flashcards, .. } => {
                let spacing = theme::active().cosmic().spacing;

                let header = header_view(spacing);
                let content = folders_view(&spacing, flashcards);

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

    /// Handles interactions for this screen
    pub fn update(&mut self, message: Message, database: &Arc<Pool<Sqlite>>) -> Action {
        match message {
            Message::None => Action::None,
            Message::AddToast(toast) => Action::AddToast(toast),
            Message::LaunchUrl(url) => {
                match open::that_detached(&url) {
                    Ok(()) => {}
                    Err(err) => {
                        eprintln!("failed to open {url:?}: {err}");
                    }
                }
                Action::None
            }
            Message::UpdateCurrentFolderId(folder_id) => {
                let State::Ready {
                    current_folder_id, ..
                } = &mut self.state
                else {
                    return Action::None;
                };

                *current_folder_id = Some(folder_id);
                Action::None
            }
            Message::LoadFlashcards => {
                let State::Ready {
                    current_folder_id, ..
                } = &self.state
                else {
                    return Action::None;
                };

                if let Some(folder_id) = *current_folder_id {
                    Action::Run(
                        Task::perform(
                            Flashcard::get_all(Arc::clone(database), folder_id),
                            Message::FlashcardsLoaded,
                        )
                        .chain(Task::done(Message::UpdateCurrentFolderId(folder_id))),
                    )
                } else {
                    Action::None
                }
            }
            Message::FlashcardsLoaded(res) => {
                match res {
                    Ok(flashcards) => {
                        // If we we're editing a flashcard don't lose it's state even if we update the main content of the page
                        let add_edit_flashcard = match &self.state {
                            State::Ready {
                                add_edit_flashcard, ..
                            } if add_edit_flashcard.id.is_some() => add_edit_flashcard.clone(),
                            _ => Box::new(Flashcard::default()),
                        };

                        self.state = State::Ready {
                            current_folder_id: None,
                            flashcards,
                            add_edit_flashcard,
                            options: FolderOptions::default(),
                        };
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        return Action::AddToast(OboeteToast::new(e));
                    }
                }
                Action::None
            }

            Message::OpenContextPage(context_page, flashcard) => {
                if let Some(flashcard) = flashcard {
                    let State::Ready {
                        add_edit_flashcard, ..
                    } = &mut self.state
                    else {
                        return Action::None;
                    };

                    *add_edit_flashcard = Box::from(flashcard);
                }

                Action::OpenContextPage(context_page)
            }

            Message::EditFlashcard => {
                let State::Ready {
                    add_edit_flashcard, ..
                } = &mut self.state
                else {
                    return Action::None;
                };

                #[allow(clippy::collapsible_if)]
                if let FlashcardField::Image { path, .. } = &mut add_edit_flashcard.front {
                    // If the image path is not in the Oboete data path we know it has not been modified so we don't need to save the image again
                    if !utils::check_path(path) {
                        let new_path = utils::save_image(path);
                        match new_path {
                            Ok(new_path) => *path = new_path,
                            Err(e) => {
                                return Action::AddToast(OboeteToast::new(e));
                            }
                        }
                    }
                }

                #[allow(clippy::collapsible_if)]
                if let FlashcardField::Image { path, .. } = &mut add_edit_flashcard.back {
                    // If the image path is not in the Oboete data path we know it has not been modified so we don't need to save the image again
                    if !utils::check_path(path) {
                        let new_path = utils::save_image(path);
                        match new_path {
                            Ok(new_path) => *path = new_path,
                            Err(e) => {
                                return Action::AddToast(OboeteToast::new(e));
                            }
                        }
                    }
                }

                Action::Run(Task::perform(
                    Flashcard::edit(Arc::clone(database), *add_edit_flashcard.clone()),
                    |res| match res {
                        Ok(_) => Message::LoadFlashcards,
                        Err(e) => {
                            eprintln!("{}", e);
                            Message::AddToast(OboeteToast::new(e))
                        }
                    },
                ))
            }
            Message::AddFlashcard => {
                let State::Ready {
                    add_edit_flashcard,
                    current_folder_id,
                    ..
                } = &mut self.state
                else {
                    return Action::None;
                };

                if let Some(folder_id) = current_folder_id {
                    // save the front flashcard image if any
                    if let FlashcardField::Image { path, .. } = &mut add_edit_flashcard.front {
                        let new_path = utils::save_image(path);
                        match new_path {
                            Ok(new_path) => *path = new_path,
                            Err(e) => {
                                return Action::AddToast(OboeteToast::new(e));
                            }
                        }
                    }

                    // save the back flashcard image if any
                    if let FlashcardField::Image { path, .. } = &mut add_edit_flashcard.back {
                        let new_path = utils::save_image(path);
                        match new_path {
                            Ok(new_path) => *path = new_path,
                            Err(e) => {
                                return Action::AddToast(OboeteToast::new(e));
                            }
                        }
                    }

                    return Action::Run(Task::perform(
                        Flashcard::add(
                            Arc::clone(database),
                            *add_edit_flashcard.clone(),
                            *folder_id,
                        ),
                        |res| match res {
                            Ok(_) => Message::LoadFlashcards,
                            Err(e) => {
                                eprintln!("{}", e);
                                Message::AddToast(OboeteToast::new(e))
                            }
                        },
                    ));
                }
                Action::None
            }
            Message::AddEditFlashcardInput(input) => {
                let State::Ready {
                    add_edit_flashcard, ..
                } = &mut self.state
                else {
                    return Action::None;
                };

                apply_flashcard_add_edit_input(input, add_edit_flashcard)
            }
            Message::ResetFlashcardStatus(flashcard_id) => Action::Run(Task::perform(
                Flashcard::reset_single_status(Arc::clone(database), flashcard_id),
                |res| match res {
                    Ok(_) => Message::LoadFlashcards,
                    Err(e) => Message::AddToast(OboeteToast::new(e)),
                },
            )),

            Message::DeleteFlashcard(flashcard_id) => {
                Action::OpenDeleteFlashcardDialog(flashcard_id)
            }

            Message::FolderOptionsInput(input) => {
                let State::Ready {
                    options,
                    current_folder_id,
                    flashcards,
                    ..
                } = &mut self.state
                else {
                    return Action::None;
                };

                if let Some(folder_id) = current_folder_id {
                    apply_folder_options_input(input, options, *folder_id, database, flashcards)
                } else {
                    Action::None
                }
            }

            Message::Study => {
                let State::Ready {
                    current_folder_id, ..
                } = &mut self.state
                else {
                    return Action::None;
                };

                if let Some(folder_id) = current_folder_id {
                    Action::StudyFolder(*folder_id)
                } else {
                    Action::None
                }
            }
        }
    }

    /// Subscriptions of this screen
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    //
    // CONTEXT PAGES
    //

    /// View of the folder/flashcard options [`ContextPage`] of the application
    pub fn options_contextpage<'a>(&'a self, spacing: Spacing) -> Element<'a, Message> {
        let State::Ready {
            flashcards,
            options,
            ..
        } = &self.state
        else {
            return text("Error").into(); // It's theoretically impossible to be here.
        };

        column![
            // CUSTOM IMPORT SECTION
            settings::view_column(vec![
                settings::section()
                    .title(fl!("folder-import"))
                    .add(
                        cosmic::widget::column::with_children(vec![
                            text::body(fl!("import-between-term-title")).into(),
                            text_input(
                                fl!("import-between-term-placeholder"),
                                &options.between_terms,
                            )
                            .on_input(|x| {
                                Message::FolderOptionsInput(FolderOptionsInput::BetweenTermsInput(
                                    x,
                                ))
                            })
                            .into(),
                            text::body(fl!("import-between-cards-title")).into(),
                            text_input(
                                fl!("import-between-cards-placeholder"),
                                &options.between_cards,
                            )
                            .on_input(|x| {
                                Message::FolderOptionsInput(FolderOptionsInput::BetweenCardsInput(
                                    x,
                                ))
                            })
                            .into(),
                            text::body(fl!("import-content-title")).into(),
                            text_input(fl!("import-content-placeholder"), &options.import_content)
                                .on_input(|x| {
                                    Message::FolderOptionsInput(
                                        FolderOptionsInput::ImportContentInput(x),
                                    )
                                })
                                .on_paste(|x| {
                                    Message::FolderOptionsInput(
                                        FolderOptionsInput::ImportContentInput(x),
                                    )
                                })
                                .into(),
                        ])
                        .spacing(spacing.space_xxs),
                    )
                    .into(),
            ]),
            row![
                Space::new(Length::Fill, Length::Shrink),
                button::text(fl!("import-button"))
                    .on_press_maybe(options.is_valid().then_some(Message::FolderOptionsInput(
                        FolderOptionsInput::CustomImport
                    ),))
                    .class(theme::Button::Suggested)
            ],
            // ANKI IMPORT SECTION
            settings::view_column(vec![
                settings::section()
                    .title(fl!("import-anki-title"))
                    .add(
                        cosmic::widget::column::with_children(vec![button::link(fl!(
                    "about-anki-importing"
                ))
                .on_press(Message::LaunchUrl(String::from(
                    "https://github.com/mariinkys/oboete/blob/main/info/ANKI_IMPORTING.md",
                )))
                .into()])
                        .spacing(spacing.space_xxs),
                    )
                    .into(),
            ]),
            Row::new()
                .push(Space::new(Length::Fill, Length::Shrink))
                .push(
                    button::text(fl!("import-button"))
                        .on_press(Message::FolderOptionsInput(FolderOptionsInput::AnkiImport))
                        .class(theme::Button::Suggested),
                ),
            // RESET SECTION
            column![
                settings::view_column(vec![
                    settings::section()
                        .title(fl!("reset-folder-flashcards-title"))
                        .into(),
                ]),
                button::text(fl!("reset-folder-flashcards-button"))
                    .class(theme::Button::Destructive)
                    .on_press_maybe((!flashcards.is_empty()).then_some(
                        Message::FolderOptionsInput(FolderOptionsInput::ResetAllStatus,)
                    ),)
            ]
            .spacing(spacing.space_xxxs),
            // EXPORT SECTION
            column![
                settings::view_column(vec![
                    settings::section()
                        .title(fl!("export-folder-flashcards-title"))
                        .into(),
                ]),
                button::text(fl!("export-folder-flashcards-button"))
                    .on_press_maybe(
                        (!flashcards.is_empty())
                            .then_some(Message::FolderOptionsInput(FolderOptionsInput::Export)),
                    )
                    .class(theme::Button::Suggested),
                button::text(fl!("export-folder-flashcards-anki-button"))
                    .on_press_maybe(
                        (!flashcards.is_empty())
                            .then_some(Message::FolderOptionsInput(FolderOptionsInput::ExportAnki)),
                    )
                    .class(theme::Button::Suggested)
            ]
            .spacing(spacing.space_xxxs)
        ]
        .spacing(spacing.space_xs)
        .into()
    }

    /// View of the upsert flashcard [`ContextPage`] of the application
    pub fn add_edit_contextpage<'a>(&'a self, spacing: Spacing) -> Element<'a, Message> {
        let State::Ready {
            add_edit_flashcard, ..
        } = &self.state
        else {
            return text("Error").into(); // It's theoretically impossible to be here.
        };

        let add_edit_button = if add_edit_flashcard.id.is_some() {
            button::text(fl!("edit"))
                .on_press_maybe(
                    add_edit_flashcard
                        .is_valid()
                        .then_some(Message::EditFlashcard),
                )
                .class(theme::Button::Suggested)
        } else {
            button::text(fl!("create"))
                .on_press_maybe(
                    add_edit_flashcard
                        .is_valid()
                        .then_some(Message::AddFlashcard),
                )
                .class(theme::Button::Suggested)
        };

        let front_input = {
            let type_selector =
                pick_list(FlashcardField::ALL, Some(&add_edit_flashcard.front), |x| {
                    Message::AddEditFlashcardInput(AddEditFlashcardInput::FrontFieldTypeChanged(x))
                })
                .width(Length::Shrink);

            let header = row![
                text::body(fl!("flashcard-front-title")).width(Length::Fill),
                type_selector,
            ]
            .spacing(spacing.space_s)
            .align_y(Alignment::Center);

            let content = match &add_edit_flashcard.front {
                FlashcardField::Text(text) => text_input(fl!("flashcard-front-placeholder"), text)
                    .on_input(|input| {
                        Message::AddEditFlashcardInput(AddEditFlashcardInput::FrontTextInput(input))
                    })
                    .into(),
                FlashcardField::Image { path, alt_text } => {
                    let image_input: Element<Message> = if path.is_empty() {
                        row![
                            text::body(fl!("select-image")).width(Length::Fill),
                            button::text(fl!("browse"))
                                .on_press(Message::AddEditFlashcardInput(
                                    AddEditFlashcardInput::FrontSelectImage
                                ))
                                .class(theme::Button::Standard)
                                .width(Length::Shrink)
                        ]
                        .align_y(Alignment::Center)
                        .into()
                    } else {
                        container(stack![
                            container(image(path).content_fit(ContentFit::Contain))
                                .center_x(Length::Fill)
                                .max_height(250.),
                            container(
                                button::icon(icons::get_handle("user-trash-full-symbolic", 18))
                                    .class(theme::Button::Destructive)
                                    .on_press(Message::AddEditFlashcardInput(
                                        AddEditFlashcardInput::DeleteFrontSelectedImage,
                                    ))
                                    .padding(8)
                            )
                            .padding(10)
                            .align_x(Horizontal::Right)
                            .align_y(Vertical::Bottom)
                            .width(Length::Fill)
                            .height(Length::Fill)
                        ])
                        .into()
                    };

                    cosmic::widget::column::with_children(vec![
                        image_input,
                        text_input(fl!("front-image-alt"), alt_text)
                            .on_input(|input| {
                                Message::AddEditFlashcardInput(
                                    AddEditFlashcardInput::FrontAltTextInput(input),
                                )
                            })
                            .into(),
                    ])
                    .spacing(spacing.space_xxs)
                    .into()
                }
            };

            cosmic::widget::column::with_children(vec![header.into(), content])
                .spacing(spacing.space_xxs)
        };

        let back_input = {
            let type_selector =
                pick_list(FlashcardField::ALL, Some(&add_edit_flashcard.back), |x| {
                    Message::AddEditFlashcardInput(AddEditFlashcardInput::BackFieldTypeChanged(x))
                })
                .width(Length::Shrink);

            let header = row![
                text::body(fl!("flashcard-back-title")).width(Length::Fill),
                type_selector
            ]
            .spacing(spacing.space_s)
            .align_y(Alignment::Center);

            let content = match &add_edit_flashcard.back {
                FlashcardField::Text(text) => text_input(fl!("flashcard-back-placeholder"), text)
                    .on_input(|input| {
                        Message::AddEditFlashcardInput(AddEditFlashcardInput::BackTextInput(input))
                    })
                    .into(),
                FlashcardField::Image { path, alt_text } => {
                    let image_input: Element<Message> = if path.is_empty() {
                        row![
                            text::body(fl!("select-image")).width(Length::Fill),
                            button::text(fl!("browse"))
                                .on_press(Message::AddEditFlashcardInput(
                                    AddEditFlashcardInput::BackSelectImage
                                ))
                                .class(theme::Button::Standard)
                                .width(Length::Shrink)
                        ]
                        .align_y(Alignment::Center)
                        .into()
                    } else {
                        container(stack![
                            container(image(path).content_fit(ContentFit::Contain))
                                .center_x(Length::Fill)
                                .max_height(250.),
                            container(
                                button::icon(icons::get_handle("user-trash-full-symbolic", 18))
                                    .class(theme::Button::Destructive)
                                    .on_press(Message::AddEditFlashcardInput(
                                        AddEditFlashcardInput::DeleteBackSelectedImage,
                                    ))
                                    .padding(8)
                            )
                            .padding(10)
                            .align_x(Horizontal::Right)
                            .align_y(Vertical::Bottom)
                            .width(Length::Fill)
                            .height(Length::Fill)
                        ])
                        .into()
                    };

                    cosmic::widget::column::with_children(vec![
                        image_input,
                        text_input(fl!("back-image-alt"), alt_text)
                            .on_input(|input| {
                                Message::AddEditFlashcardInput(
                                    AddEditFlashcardInput::BackAltTextInput(input),
                                )
                            })
                            .into(),
                    ])
                    .spacing(spacing.space_xxs)
                    .into()
                }
            };

            cosmic::widget::column::with_children(vec![header.into(), content])
                .spacing(spacing.space_xxs)
        };

        column![
            settings::view_column(vec![
                settings::section()
                    .title(fl!("flashcard-options"))
                    .add(
                        cosmic::widget::column::with_children(vec![
                            front_input.into(),
                            back_input.into(),
                        ])
                        .spacing(spacing.space_s)
                    )
                    .into(),
            ]),
            row![Space::new(Length::Fill, Length::Shrink), add_edit_button],
            settings::view_column(vec![
                settings::section()
                    .title(fl!("reset-flashcard-title"))
                    .into(),
            ]),
            row![
                text(format!(
                    "{}: {}",
                    fl!("current-flashcard-status"),
                    add_edit_flashcard.status
                ))
                .width(Length::Fill),
                button::text(fl!("reset-flashcard-button"))
                    .on_press_maybe(add_edit_flashcard.id.is_some().then_some(
                        Message::ResetFlashcardStatus(add_edit_flashcard.id.unwrap_or_default(),)
                    ))
                    .class(theme::Button::Destructive)
            ]
            .align_y(Alignment::Center)
        ]
        .spacing(spacing.space_xxs)
        .into()
    }
}

//
// VIEWS
//

/// View of the header of this screen
fn header_view<'a>(spacing: Spacing) -> Element<'a, Message> {
    let new_flashcard_button = button::icon(icons::get_handle("list-add-symbolic", 18))
        .class(theme::Button::Suggested)
        .on_press(Message::OpenContextPage(
            ContextPage::AddEditFlashcard,
            Some(Flashcard::default()),
        ));

    let study_button = button::text(fl!("study"))
        .class(theme::Button::Suggested)
        .on_press(Message::Study);

    let options_button = button::icon(icons::get_handle("emblem-system-symbolic", 18))
        .class(theme::Button::Suggested)
        .on_press(Message::OpenContextPage(
            ContextPage::FolderContentOptions,
            None,
        ));

    cosmic::widget::row::with_capacity(5)
        .align_y(Alignment::Center)
        .spacing(spacing.space_s)
        .padding([spacing.space_none, spacing.space_xxs])
        .push(text::title3(fl!("flashcards")).width(Length::Shrink))
        .push(options_button)
        .push(Space::new(Length::Fill, Length::Shrink))
        .push(study_button)
        .push(new_flashcard_button)
        .into()
}

/// View of the contents of this screen
fn folders_view<'a>(spacing: &Spacing, flashcards: &'a [Flashcard]) -> Element<'a, Message> {
    let content: Element<'a, Message> = if flashcards.is_empty() {
        text(fl!("empty-flashcards-page")).into()
    } else {
        let mut flashcards_list = list::list_column().style(theme::Container::Card);

        for flashcard in flashcards {
            let front_text = match &flashcard.front {
                FlashcardField::Text(t) => t,
                FlashcardField::Image { alt_text, .. } => alt_text,
            };

            flashcards_list = flashcards_list.add(
                row![
                    button::icon(icons::get_handle("edit-symbolic", 18))
                        .class(theme::Button::Standard)
                        .width(Length::Shrink)
                        .on_press(Message::OpenContextPage(
                            ContextPage::AddEditFlashcard,
                            Some(flashcard.clone())
                        )),
                    text(front_text)
                        .align_y(Vertical::Center)
                        .align_x(Horizontal::Left)
                        .width(Length::Fill),
                    container(
                        pill(flashcard.status.to_string()).color(flashcard.status.get_color())
                    )
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Right)
                    .width(Length::Fill),
                    button::icon(icons::get_handle("user-trash-full-symbolic", 18))
                        .class(theme::Button::Destructive)
                        .on_press(Message::DeleteFlashcard(flashcard.clone()))
                ]
                .width(Length::Fill)
                .align_y(Alignment::Center)
                .spacing(spacing.space_s),
            );
        }

        flashcards_list.into()
    };

    scrollable(
        container(content)
            .align_x(Horizontal::Center)
            .width(Length::Fill),
    )
    .into()
}

//
// HELPERS
//

/// Given the [`AddEditFlashcardInput`] apply the appropiate state changes
fn apply_flashcard_add_edit_input(
    input: AddEditFlashcardInput,
    flashcard: &mut Flashcard,
) -> Action {
    match input {
        AddEditFlashcardInput::FrontFieldTypeChanged(flashcard_field) => {
            flashcard.front = flashcard_field;
        }
        AddEditFlashcardInput::FrontTextInput(input) => {
            flashcard.front = FlashcardField::Text(input);
        }
        AddEditFlashcardInput::FrontSelectImage => {
            return Action::Run(Task::perform(
                async move {
                    let result = SelectedFiles::open_file()
                        .title("Select Image")
                        .accept_label("Open")
                        .modal(true)
                        .multiple(false)
                        .filter(
                            FileFilter::new("Image Files")
                                .glob("*.png")
                                .glob("*.jpeg")
                                .glob("*.jpg"),
                        )
                        .send()
                        .await
                        .unwrap()
                        .response();

                    if let Ok(result) = result {
                        result
                            .uris()
                            .iter()
                            .map(|file| file.path().to_string())
                            .collect::<Vec<String>>()
                            .first()
                            .cloned()
                            .unwrap_or(String::new())
                    } else {
                        String::new()
                    }
                },
                |res| {
                    Message::AddEditFlashcardInput(AddEditFlashcardInput::FrontImageSelected(res))
                },
            ));
        }
        AddEditFlashcardInput::FrontImageSelected(selected_path) =>
        {
            #[allow(clippy::collapsible_if)]
            if !selected_path.is_empty() {
                if let FlashcardField::Image { path, .. } = &mut flashcard.front {
                    *path = percent_decode(selected_path.as_bytes())
                        .decode_utf8_lossy()
                        .to_string();
                };
            }
        }
        AddEditFlashcardInput::DeleteFrontSelectedImage => {
            #[allow(clippy::collapsible_if)]
            if let FlashcardField::Image { path, .. } = &mut flashcard.front {
                if flashcard.id.is_some() && utils::check_path(path) {
                    let path_clone = path.clone();
                    *path = String::new();
                    return Action::Run(Task::perform(utils::delete_image(path_clone), |res| {
                        match res {
                            Ok(_) => Message::None,
                            Err(e) => Message::AddToast(OboeteToast::new(e)),
                        }
                    }));
                } else {
                    *path = String::new();
                }
            };
        }
        AddEditFlashcardInput::FrontAltTextInput(input) => {
            if let FlashcardField::Image { alt_text, .. } = &mut flashcard.front {
                *alt_text = input;
            };
        }

        AddEditFlashcardInput::BackFieldTypeChanged(flashcard_field) => {
            flashcard.back = flashcard_field;
        }
        AddEditFlashcardInput::BackTextInput(input) => {
            flashcard.back = FlashcardField::Text(input);
        }
        AddEditFlashcardInput::BackSelectImage => {
            return Action::Run(Task::perform(
                async move {
                    let result = SelectedFiles::open_file()
                        .title("Select Image")
                        .accept_label("Open")
                        .modal(true)
                        .multiple(false)
                        .filter(
                            FileFilter::new("Image Files")
                                .glob("*.png")
                                .glob("*.jpeg")
                                .glob("*.jpg"),
                        )
                        .send()
                        .await
                        .unwrap()
                        .response();

                    if let Ok(result) = result {
                        result
                            .uris()
                            .iter()
                            .map(|file| file.path().to_string())
                            .collect::<Vec<String>>()
                            .first()
                            .cloned()
                            .unwrap_or(String::new())
                    } else {
                        String::new()
                    }
                },
                |res| Message::AddEditFlashcardInput(AddEditFlashcardInput::BackImageSelected(res)),
            ));
        }
        AddEditFlashcardInput::BackImageSelected(selected_path) =>
        {
            #[allow(clippy::collapsible_if)]
            if !selected_path.is_empty() {
                if let FlashcardField::Image { path, .. } = &mut flashcard.back {
                    *path = percent_decode(selected_path.as_bytes())
                        .decode_utf8_lossy()
                        .to_string();
                };
            }
        }
        AddEditFlashcardInput::DeleteBackSelectedImage => {
            #[allow(clippy::collapsible_if)]
            if let FlashcardField::Image { path, .. } = &mut flashcard.back {
                if flashcard.id.is_some() && utils::check_path(path) {
                    let path_clone = path.clone();
                    *path = String::new();
                    return Action::Run(Task::perform(utils::delete_image(path_clone), |res| {
                        match res {
                            Ok(_) => Message::None,
                            Err(e) => Message::AddToast(OboeteToast::new(e)),
                        }
                    }));
                } else {
                    *path = String::new();
                }
            };
        }
        AddEditFlashcardInput::BackAltTextInput(input) => {
            if let FlashcardField::Image { alt_text, .. } = &mut flashcard.back {
                *alt_text = input;
            };
        }
    }

    Action::None
}

/// Given the [`FolderOptionsInput`] apply the appropiate state changes
fn apply_folder_options_input(
    input: FolderOptionsInput,
    options: &mut FolderOptions,
    folder_id: i32,
    database: &Arc<Pool<Sqlite>>,
    flashcards: &[Flashcard],
) -> Action {
    match input {
        FolderOptionsInput::ImportContentInput(input) => {
            options.import_content = input;
        }
        FolderOptionsInput::BetweenCardsInput(input) => {
            options.between_cards = input;
        }
        FolderOptionsInput::BetweenTermsInput(input) => {
            options.between_terms = input;
        }
        FolderOptionsInput::CustomImport => {
            let content = utils::parse_import_content(
                &options.between_cards,
                &options.between_terms,
                &options.import_content,
            );
            return Action::Run(Task::perform(
                Flashcard::add_bulk(Arc::clone(database), content, folder_id),
                |res| match res {
                    Ok(_) => Message::LoadFlashcards,
                    Err(e) => Message::AddToast(OboeteToast::new(e)),
                },
            ));
        }

        FolderOptionsInput::AnkiImport => {
            return Action::Run(Task::perform(
                async move {
                    let result = SelectedFiles::open_file()
                        .title("Select Anki File")
                        .accept_label("Open")
                        .modal(true)
                        .multiple(false)
                        .filter(FileFilter::new("TXT File").glob("*.txt"))
                        .send()
                        .await
                        .unwrap()
                        .response();

                    if let Ok(result) = result {
                        result
                            .uris()
                            .iter()
                            .map(|file| file.path().to_string())
                            .collect::<Vec<String>>()
                            .first()
                            .cloned()
                            .unwrap_or(String::new())
                    } else {
                        String::new()
                    }
                },
                |res| Message::FolderOptionsInput(FolderOptionsInput::CompleteAnkiImport(res)),
            ));
        }
        FolderOptionsInput::CompleteAnkiImport(file_path) => {
            let parsed_content_res = utils::parse_ankifile(&file_path);

            if let Ok(content) = parsed_content_res {
                return Action::Run(Task::perform(
                    Flashcard::add_bulk(Arc::clone(database), content, folder_id),
                    |res| match res {
                        Ok(_) => Message::LoadFlashcards,
                        Err(e) => Message::AddToast(OboeteToast::new(e)),
                    },
                ));
            } else {
                return Action::AddToast(OboeteToast::new("No content found"));
            }
        }

        FolderOptionsInput::ResetAllStatus => {
            return Action::Run(Task::perform(
                Flashcard::reset_all_status(Arc::clone(database), folder_id),
                |res| match res {
                    Ok(_) => Message::LoadFlashcards,
                    Err(e) => Message::AddToast(OboeteToast::new(e)),
                },
            ));
        }

        FolderOptionsInput::Export => {
            return Action::Run(Task::perform(
                async move {
                    let result = SelectedFiles::save_file()
                        .title("Save Export File")
                        .accept_label("Save")
                        .modal(true)
                        .filter(FileFilter::new("TXT File").glob("*.txt"))
                        .send()
                        .await
                        .unwrap()
                        .response();

                    if let Ok(result) = result {
                        result
                            .uris()
                            .iter()
                            .map(|file| file.path().to_string())
                            .collect::<Vec<String>>()
                            .first()
                            .cloned()
                            .unwrap_or(String::new())
                    } else {
                        String::new()
                    }
                },
                |res| Message::FolderOptionsInput(FolderOptionsInput::CompleteExport(res)),
            ));
        }
        FolderOptionsInput::CompleteExport(file_path) => {
            let res = utils::export_flashcards(&file_path, flashcards);
            if let Err(e) = res {
                eprintln!("{}", e);
                return Action::AddToast(OboeteToast::new(e));
            }
            return Action::Run(Task::done(Message::LoadFlashcards));
        }
        FolderOptionsInput::ExportAnki => {
            return Action::Run(Task::perform(
                async move {
                    let result = SelectedFiles::save_file()
                        .title("Save Export File")
                        .accept_label("Save")
                        .modal(true)
                        .filter(FileFilter::new("TXT File").glob("*.txt"))
                        .send()
                        .await
                        .unwrap()
                        .response();

                    if let Ok(result) = result {
                        result
                            .uris()
                            .iter()
                            .map(|file| file.path().to_string())
                            .collect::<Vec<String>>()
                            .first()
                            .cloned()
                            .unwrap_or(String::new())
                    } else {
                        String::new()
                    }
                },
                |res| Message::FolderOptionsInput(FolderOptionsInput::CompleteAnkiExport(res)),
            ));
        }
        FolderOptionsInput::CompleteAnkiExport(file_path) => {
            let res = utils::export_flashcards_anki(&file_path, flashcards);
            if let Err(e) = res {
                eprintln!("{}", e);
                return Action::AddToast(OboeteToast::new(e));
            }
            return Action::Run(Task::done(Message::LoadFlashcards));
        }
    }
    Action::None
}
