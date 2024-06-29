use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Color, Length,
    },
    theme,
    widget::{self},
    Apply, Element,
};

use crate::{
    core::icon_cache::IconCache,
    fl,
    models::Flashcard,
    utils::{parse_ankifile, parse_import_content, select_random_flashcard},
};
pub struct Flashcards {
    pub current_folder_id: i32,
    pub flashcards: Vec<Flashcard>,
    pub new_edit_flashcard: CreateEditFlashcardState,
    pub currently_studying_flashcard: Flashcard,
    pub currently_studying_flashcard_side: CurrentFlashcardSide,
    pub options_page_input: OptionsContextPageInputState,
}

pub struct CreateEditFlashcardState {
    id: Option<i32>,
    front: String,
    back: String,
    status: i32,
}

impl CreateEditFlashcardState {
    pub fn new() -> CreateEditFlashcardState {
        CreateEditFlashcardState {
            id: None,
            front: String::new(),
            back: String::new(),
            status: 0,
        }
    }
}

pub struct OptionsContextPageInputState {
    pub between_terms: String,
    pub between_cards: String,
    pub import_content: String,
}

impl OptionsContextPageInputState {
    pub fn new() -> OptionsContextPageInputState {
        OptionsContextPageInputState {
            between_terms: String::new(),
            between_cards: String::new(),
            import_content: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Upsert,
    Upserted,
    Load,
    LoadedSingle(Flashcard),
    SetFlashcards(Vec<Flashcard>),
    ToggleCreatePage(Option<Flashcard>),
    StudyFlashcards,
    ContextPageFrontInput(String),
    ContextPageBackInput(String),
    UpdateFlashcardStatus(Flashcard, StudyActions),
    UpdatedStatus(Vec<Flashcard>),
    SwapFlashcardSide,
    Delete(Option<i32>),
    ToggleOptionsPage,
    OptionsPageInput(OptionsContextPageInputActions),
    Import,
    RestartSingleFlashcardStatus(Option<i32>),
    RestartFolderFlashcardStatus,
    OpenAnkiFileSelection,
    OpenAnkiFileResult(Vec<String>),
}

pub enum Command {
    //The i32 is the Folder Id
    LoadFlashcards(i32),
    ToggleCreateFlashcardPage(Option<Flashcard>),
    UpsertFlashcard(Flashcard),
    OpenStudyFolderFlashcardsPage,
    UpdateFlashcardStatus(Flashcard),
    DeleteFlashcard(Option<i32>),
    ToggleOptionsPage,
    ImportFlashcards(Vec<Flashcard>),
    RestartSingleFlashcardStatus(Option<i32>),
    RestartFolderFlashcardStatus(i32),
    OpenAnkiFileSelection,
}

#[derive(Debug, Clone)]
pub enum StudyActions {
    Bad,
    Ok,
    Good,
}

#[derive(Debug, Clone)]
pub enum CurrentFlashcardSide {
    Front,
    Back,
}

#[derive(Debug, Clone)]
pub enum OptionsContextPageInputActions {
    BetweenTerms(String),
    BetweenCards(String),
    ImportContent(String),
}

impl Flashcards {
    pub fn new() -> Self {
        Self {
            current_folder_id: 0,
            flashcards: Vec::new(),
            currently_studying_flashcard: Flashcard {
                id: None,
                front: String::from("Error"),
                back: String::from("Error"),
                status: 0,
            },
            new_edit_flashcard: CreateEditFlashcardState::new(),
            currently_studying_flashcard_side: CurrentFlashcardSide::Front,
            options_page_input: OptionsContextPageInputState::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = Vec::new();

        match message {
            Message::Upsert => commands.push(Command::UpsertFlashcard(Flashcard {
                id: self.new_edit_flashcard.id,
                front: self.new_edit_flashcard.front.to_string(),
                back: self.new_edit_flashcard.back.to_string(),
                status: self.new_edit_flashcard.status,
            })),
            Message::Upserted => {
                self.new_edit_flashcard = CreateEditFlashcardState::new();
                self.options_page_input = OptionsContextPageInputState::new();

                commands.push(Command::LoadFlashcards(self.current_folder_id))
            }
            Message::LoadedSingle(flashcard) => {
                self.new_edit_flashcard = CreateEditFlashcardState {
                    id: flashcard.id,
                    front: flashcard.front,
                    back: flashcard.back,
                    status: flashcard.status,
                };
            }
            Message::SetFlashcards(flashcards) => self.flashcards = flashcards,
            Message::ToggleCreatePage(flashcard) => {
                if flashcard.is_none() {
                    self.new_edit_flashcard = CreateEditFlashcardState::new();
                }

                commands.push(Command::ToggleCreateFlashcardPage(flashcard))
            }
            Message::StudyFlashcards => commands.push(Command::OpenStudyFolderFlashcardsPage),
            Message::ContextPageFrontInput(value) => self.new_edit_flashcard.front = value,
            Message::ContextPageBackInput(value) => self.new_edit_flashcard.back = value,
            Message::UpdateFlashcardStatus(mut flashcard, action) => {
                match action {
                    StudyActions::Bad => flashcard.status = 1,
                    StudyActions::Ok => flashcard.status = 2,
                    StudyActions::Good => flashcard.status = 3,
                }

                commands.push(Command::UpdateFlashcardStatus(flashcard))
            }
            Message::UpdatedStatus(flashcards) => {
                self.flashcards = flashcards;
                self.currently_studying_flashcard_side = CurrentFlashcardSide::Front;
                self.currently_studying_flashcard = select_random_flashcard(&self.flashcards)
                    .unwrap_or(Flashcard::new_error_variant());
            }
            Message::SwapFlashcardSide => match self.currently_studying_flashcard_side {
                CurrentFlashcardSide::Front => {
                    self.currently_studying_flashcard_side = CurrentFlashcardSide::Back
                }
                CurrentFlashcardSide::Back => {
                    self.currently_studying_flashcard_side = CurrentFlashcardSide::Front
                }
            },
            Message::Delete(flashcard_id) => commands.push(Command::DeleteFlashcard(flashcard_id)),
            Message::Load => commands.push(Command::LoadFlashcards(self.current_folder_id)),
            Message::ToggleOptionsPage => commands.push(Command::ToggleOptionsPage),
            Message::OptionsPageInput(input) => match input {
                OptionsContextPageInputActions::BetweenTerms(value) => {
                    self.options_page_input.between_terms = value
                }
                OptionsContextPageInputActions::BetweenCards(value) => {
                    self.options_page_input.between_cards = value
                }
                OptionsContextPageInputActions::ImportContent(value) => {
                    self.options_page_input.import_content = value
                }
            },
            Message::Import => {
                let content = parse_import_content(
                    &self.options_page_input.between_cards,
                    &self.options_page_input.between_terms,
                    &self.options_page_input.import_content,
                );
                commands.push(Command::ImportFlashcards(content))
            }
            Message::RestartSingleFlashcardStatus(flashcard_id) => {
                commands.push(Command::RestartSingleFlashcardStatus(flashcard_id))
            }
            Message::RestartFolderFlashcardStatus => commands.push(
                Command::RestartFolderFlashcardStatus(self.current_folder_id),
            ),
            Message::OpenAnkiFileSelection => commands.push(Command::OpenAnkiFileSelection),
            Message::OpenAnkiFileResult(open_result) => {
                for path in open_result {
                    let flashcards = parse_ankifile(&path);
                    match flashcards {
                        Ok(flashcards) => commands.push(Command::ImportFlashcards(flashcards)),
                        Err(err) => println!("{:?}", err), //TODO: Error handling?
                    }
                }
            }
        }

        commands
    }

    fn flashcard_header_row(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        //TODO: Replace Text with IconCache::get("add-symbolic", 18) - For now it causes visual issues when the page is empty...
        let new_flashcard_button = widget::button(widget::text(fl!("new")))
            .style(theme::Button::Suggested)
            .padding(spacing.space_xxs)
            .on_press(Message::ToggleCreatePage(None));

        //TODO: IconCache::get("menu-vertical-symbolic", 18) - For now it causes visual issues when the page is empty...
        let flashcard_options_button = widget::button(widget::text(fl!("options")))
            .style(theme::Button::Standard)
            .padding(spacing.space_xxs)
            .on_press(Message::ToggleOptionsPage);

        let study_button = if self.flashcards.is_empty() == false {
            widget::button(widget::text(fl!("study")))
                .style(theme::Button::Suggested)
                .padding(spacing.space_xxs)
                .on_press(Message::StudyFlashcards)
        } else {
            widget::button(widget::text(fl!("study")))
                .style(theme::Button::Suggested)
                .padding(spacing.space_xxs)
        };

        widget::row::with_capacity(3)
            .align_items(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text::title3(fl!("flashcards")).width(Length::Fill))
            .push(study_button)
            .push(new_flashcard_button)
            .push(flashcard_options_button)
            .into()
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.flashcards.is_empty() == false {
            let mut flashcards = widget::list::list_column()
                .style(theme::Container::ContextDrawer)
                .spacing(spacing.space_xxxs)
                .padding([spacing.space_none, spacing.space_xxs]);

            for flashcard in &self.flashcards {
                let edit_button = widget::button(IconCache::get("edit-button-symbolic", 18))
                    .padding(spacing.space_xxs)
                    .style(theme::Button::Standard)
                    .on_press(Message::ToggleCreatePage(Some(flashcard.clone())));

                let delete_button = widget::button(IconCache::get("user-trash-full-symbolic", 18))
                    .padding(spacing.space_xxs)
                    .style(theme::Button::Destructive)
                    .on_press(Message::Delete(flashcard.id));

                //TODO: Custom Button to make it look like a badge
                let badge = widget::text(match flashcard.status {
                    1 => format!("{}     ", fl!("bad-status")), // High chance (status = 1 = flashcard Bad)
                    2 => format!("{}     ", fl!("ok-status")), // Medium chance (status = 2 = flashcard Ok)
                    3 => format!("{}     ", fl!("good-status")), // Low chance (status = 3 = flashcard Good)
                    _ => String::new(), // Default chance for other statuses
                })
                .vertical_alignment(Vertical::Center)
                .horizontal_alignment(Horizontal::Left)
                .width(Length::Shrink);

                let flashcard_front = widget::text(flashcard.front.clone())
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Left)
                    .width(Length::Fill);

                let row = widget::row::with_capacity(2)
                    .align_items(Alignment::Center)
                    .spacing(spacing.space_xxs)
                    .padding([spacing.space_xxxs, spacing.space_xxs])
                    .push(flashcard_front)
                    .push(badge)
                    .push(delete_button)
                    .push(edit_button);

                flashcards = flashcards.add(row);
            }

            widget::column::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(self.flashcard_header_row())
                .push(flashcards)
                .apply(widget::container)
                .height(Length::Shrink)
                .apply(widget::scrollable)
                .height(Length::Fill)
                .into()
        } else {
            widget::column::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(self.flashcard_header_row())
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

    /// The create or edit flashcard context page for this app.
    pub fn create_edit_flashcard_contextpage(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        widget::settings::view_column(vec![
            widget::settings::view_section(fl!("flashcard-options"))
                .add(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("flashcard-front-title")).into(),
                        widget::text_input(
                            fl!("flashcard-front-placeholder"),
                            &self.new_edit_flashcard.front,
                        )
                        .on_input(Message::ContextPageFrontInput)
                        .into(),
                    ])
                    .spacing(spacing.space_xxs)
                    .padding([0, 15, 0, 15]),
                )
                .add(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("flashcard-back-title")).into(),
                        widget::text_input(
                            fl!("flashcard-back-placeholder"),
                            &self.new_edit_flashcard.back,
                        )
                        .on_input(Message::ContextPageBackInput)
                        .into(),
                    ])
                    .spacing(spacing.space_xxs)
                    .padding([0, 15, 0, 15]),
                )
                .add(match self.new_edit_flashcard.id {
                    Some(_id) => {
                        if self.new_edit_flashcard.front.is_empty() == false
                            && self.new_edit_flashcard.back.is_empty() == false
                        {
                            widget::button(
                                widget::text(fl!("edit"))
                                    .horizontal_alignment(
                                        cosmic::iced::alignment::Horizontal::Center,
                                    )
                                    .width(Length::Fill),
                            )
                            .on_press(Message::Upsert)
                            .style(theme::Button::Suggested)
                            .padding([10, 0, 10, 0])
                            .width(Length::Fill)
                        } else {
                            widget::button(
                                widget::text(fl!("edit"))
                                    .horizontal_alignment(
                                        cosmic::iced::alignment::Horizontal::Center,
                                    )
                                    .width(Length::Fill),
                            )
                            .style(theme::Button::Suggested)
                            .padding([10, 0, 10, 0])
                            .width(Length::Fill)
                        }
                    }
                    None => {
                        if self.new_edit_flashcard.front.is_empty() == false
                            && self.new_edit_flashcard.back.is_empty() == false
                        {
                            widget::button(
                                widget::text(fl!("create"))
                                    .horizontal_alignment(
                                        cosmic::iced::alignment::Horizontal::Center,
                                    )
                                    .width(Length::Fill),
                            )
                            .on_press(Message::Upsert)
                            .style(theme::Button::Suggested)
                            .padding([10, 0, 10, 0])
                            .width(Length::Fill)
                        } else {
                            widget::button(
                                widget::text(fl!("create"))
                                    .horizontal_alignment(
                                        cosmic::iced::alignment::Horizontal::Center,
                                    )
                                    .width(Length::Fill),
                            )
                            .style(theme::Button::Suggested)
                            .padding([10, 0, 10, 0])
                            .width(Length::Fill)
                        }
                    }
                })
                .into(),
            widget::settings::view_section(fl!("reset-flashcard-title"))
                .add(
                    widget::button(
                        widget::text(fl!("reset-flashcard-button"))
                            .horizontal_alignment(cosmic::iced::alignment::Horizontal::Center)
                            .width(Length::Fill),
                    )
                    .on_press(Message::RestartSingleFlashcardStatus(
                        self.new_edit_flashcard.id,
                    ))
                    .style(theme::Button::Destructive)
                    .padding([10, 0, 10, 0])
                    .width(Length::Fill),
                )
                .into(),
        ])
        .into()
    }

    pub fn view_study_page(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let flashcard_container = widget::container(
            widget::button(
                widget::Text::new(match self.currently_studying_flashcard_side {
                    CurrentFlashcardSide::Front => &self.currently_studying_flashcard.front,
                    CurrentFlashcardSide::Back => &self.currently_studying_flashcard.back,
                })
                .size(spacing.space_xxl)
                .width(Length::Fill)
                .height(Length::Fill)
                .vertical_alignment(Vertical::Center)
                .horizontal_alignment(Horizontal::Center),
            )
            .on_press(Message::SwapFlashcardSide)
            .style(button_style(false, false, ButtonStyle::NoHover))
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .style(theme::Container::ContextDrawer)
        .width(Length::Fill)
        .height(Length::Fill);

        let options_row = widget::row::with_capacity(3)
            .push(
                widget::button(
                    widget::Text::new(fl!("bad-status"))
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .on_press(Message::UpdateFlashcardStatus(
                    self.currently_studying_flashcard.clone(),
                    StudyActions::Bad,
                ))
                .style(button_style(false, false, ButtonStyle::BadButton))
                .height(Length::Fixed(60.0))
                .width(Length::Fill),
            )
            .push(
                widget::button(
                    widget::Text::new(fl!("ok-status"))
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .on_press(Message::UpdateFlashcardStatus(
                    self.currently_studying_flashcard.clone(),
                    StudyActions::Ok,
                ))
                .style(button_style(false, false, ButtonStyle::OkButton))
                .height(Length::Fixed(60.0))
                .width(Length::Fill),
            )
            .push(
                widget::button(
                    widget::Text::new(fl!("good-status"))
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .on_press(Message::UpdateFlashcardStatus(
                    self.currently_studying_flashcard.clone(),
                    StudyActions::Good,
                ))
                .style(button_style(false, false, ButtonStyle::GoodButton))
                .height(Length::Fixed(60.0))
                .width(Length::Fill),
            )
            .align_items(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .width(Length::Fill);

        widget::Column::new()
            .push(flashcard_container)
            .push(options_row)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .into()
    }

    // The flashcard options context page for this app.
    pub fn flashcard_options_contextpage(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        widget::settings::view_column(vec![
            widget::settings::view_section(fl!("folder-import"))
                .add(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("import-between-term-title")).into(),
                        widget::text_input(
                            fl!("import-between-term-placeholder"),
                            &self.options_page_input.between_terms,
                        )
                        .on_input(|value| {
                            Message::OptionsPageInput(OptionsContextPageInputActions::BetweenTerms(
                                value,
                            ))
                        })
                        .into(),
                    ])
                    .spacing(spacing.space_xxs)
                    .padding([0, 15, 0, 15]),
                )
                .add(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("import-between-cards-title")).into(),
                        widget::text_input(
                            fl!("import-between-cards-placeholder"),
                            &self.options_page_input.between_cards,
                        )
                        .on_input(|value| {
                            Message::OptionsPageInput(OptionsContextPageInputActions::BetweenCards(
                                value,
                            ))
                        })
                        .into(),
                    ])
                    .spacing(spacing.space_xxs)
                    .padding([0, 15, 0, 15]),
                )
                .add(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("import-content-title")).into(),
                        //TODO: Can we Increase the Height of the Input without touching the width?
                        widget::text_input(
                            fl!("import-content-placeholder"),
                            &self.options_page_input.import_content,
                        )
                        .on_input(|value| {
                            Message::OptionsPageInput(
                                OptionsContextPageInputActions::ImportContent(value),
                            )
                        })
                        .into(),
                    ])
                    .spacing(spacing.space_xxs)
                    .padding([0, 15, 0, 15]),
                )
                .add(
                    if self.options_page_input.import_content.is_empty() == false
                        && self.options_page_input.between_cards.is_empty() == false
                        && self.options_page_input.between_terms.is_empty() == false
                    {
                        widget::button(
                            widget::text(fl!("import-button"))
                                .horizontal_alignment(cosmic::iced::alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::Import)
                        .style(theme::Button::Suggested)
                        .padding([10, 0, 10, 0])
                        .width(Length::Fill)
                    } else {
                        widget::button(
                            widget::text(fl!("import-button"))
                                .horizontal_alignment(cosmic::iced::alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .style(theme::Button::Suggested)
                        .padding([10, 0, 10, 0])
                        .width(Length::Fill)
                    },
                )
                .into(),
            widget::settings::view_section(fl!("import-anki-title"))
                .add(
                    widget::button(
                        widget::text(fl!("import-anki-button"))
                            .horizontal_alignment(cosmic::iced::alignment::Horizontal::Center)
                            .width(Length::Fill),
                    )
                    .on_press(Message::OpenAnkiFileSelection)
                    .style(theme::Button::Suggested)
                    .padding([10, 0, 10, 0])
                    .width(Length::Fill),
                )
                .into(),
            widget::settings::view_section(fl!("reset-folder-flashcards-title"))
                .add(
                    widget::button(
                        widget::text(fl!("reset-folder-flashcards-button"))
                            .horizontal_alignment(cosmic::iced::alignment::Horizontal::Center)
                            .width(Length::Fill),
                    )
                    .on_press(Message::RestartFolderFlashcardStatus)
                    .style(theme::Button::Destructive)
                    .padding([10, 0, 10, 0])
                    .width(Length::Fill),
                )
                .into(),
        ])
        .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ButtonStyle {
    NoHover,
    OkButton,
    GoodButton,
    BadButton,
}

fn button_appearance(
    theme: &theme::Theme,
    _selected: bool,
    _focused: bool,
    _accent: bool,
    style: ButtonStyle,
) -> widget::button::Appearance {
    let cosmic = theme.cosmic();
    let mut appearance = widget::button::Appearance::new();

    // Sample Code from cosmic-files (src/tab.rs I belive)
    // if selected {
    //     if accent {
    //         appearance.background = Some(Color::from(cosmic.on_accent_color()).into());
    //         appearance.icon_color = Some(Color::from(cosmic.on_accent_color()));
    //         appearance.text_color = Some(Color::from(cosmic.on_accent_color()));
    //     } else {
    //         appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
    //     }
    // }
    // if focused && accent {
    //     appearance.outline_width = 1.0;
    //     appearance.outline_color = Color::from(cosmic.accent_color());
    //     appearance.border_width = 2.0;
    //     appearance.border_color = Color::TRANSPARENT;
    // }
    // appearance.border_radius = cosmic.radius_s().into();

    appearance.border_radius = cosmic.radius_xs().into();
    appearance.icon_color = Some(Color::from(cosmic.on_accent_color()));
    appearance.outline_width = 1.0;
    appearance.border_width = 2.0;

    if style != ButtonStyle::NoHover {
        appearance.text_color = Some(Color::from(cosmic.on_accent_color()));
    }

    let custom_bg_color = match style {
        // Orange
        ButtonStyle::OkButton => Color {
            r: 245.0 / 255.0,
            g: 188.0 / 255.0,
            b: 66.0 / 255.0,
            a: 0.75,
        },
        // Green
        ButtonStyle::GoodButton => Color {
            r: 21.0 / 255.0,
            g: 191.0 / 255.0,
            b: 89.0 / 255.0,
            a: 0.75,
        },
        // Red
        ButtonStyle::BadButton => Color {
            r: 191.0 / 255.0,
            g: 57.0 / 255.0,
            b: 57.0 / 255.0,
            a: 0.75,
        },
        ButtonStyle::NoHover => Color::from(cosmic.bg_color()),
    };

    let custom_border_color = match style {
        // Darker Orange
        ButtonStyle::OkButton => Color {
            r: 250.0 / 255.0,
            g: 146.0 / 255.0,
            b: 12.0 / 255.0,
            a: 1.0,
        },
        // Brighter Green
        ButtonStyle::GoodButton => Color {
            r: 10.0 / 255.0,
            g: 209.0 / 255.0,
            b: 90.0 / 255.0,
            a: 1.0,
        },
        // Darker Red
        ButtonStyle::BadButton => Color {
            r: 107.0 / 255.0,
            g: 7.0 / 255.0,
            b: 7.0 / 255.0,
            a: 1.0,
        },
        ButtonStyle::NoHover => Color::from(cosmic.bg_color()),
    };

    appearance.background = Some(Color::from(custom_bg_color).into());
    appearance.border_color = Color::from(custom_border_color);

    appearance
}

fn button_style(selected: bool, accent: bool, style: ButtonStyle) -> theme::Button {
    theme::Button::Custom {
        active: Box::new(move |focused, theme| {
            button_appearance(theme, selected, focused, accent, style)
        }),
        disabled: Box::new(move |theme| button_appearance(theme, selected, false, accent, style)),
        hovered: Box::new(move |focused, theme| {
            button_appearance(theme, selected, focused, accent, style)
        }),
        pressed: Box::new(move |focused, theme| {
            button_appearance(theme, selected, focused, accent, style)
        }),
    }
}
