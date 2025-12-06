// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use cosmic::cosmic_theme::Spacing;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Color, ContentFit, Font, Length, Subscription};
use cosmic::iced_widget::{column, row, stack};
use cosmic::widget::{button, container, image, mouse_area, text, tooltip};
use cosmic::{Element, Task, theme};
use sqlx::{Pool, Sqlite};

use crate::app::core::models::flashcard::{Flashcard, FlashcardField, FlashcardStatus};
use crate::app::core::utils::fsrs_scheduler::FSRSScheduler;
use crate::app::core::utils::{self, OboeteToast};
use crate::fl;

/// Screen [`State`] holder
pub struct StudyScreen {
    state: State,
}

/// The different states this screen can be in
enum State {
    Loading,
    Ready {
        scheduler: Box<FSRSScheduler>,
        current_folder_id: Option<i32>,
        flashcards: Vec<Flashcard>,
        studying_flashcard: StudyingFlashcard,
        current_index: usize, // Track position in due cards
        current_mode: PracticeMode,
    },
}

/// The studymode the user is currently in
enum PracticeMode {
    Fsrs,
    Study,
}

/// Holds the state of the currently studying [`Flashcard`]
struct StudyingFlashcard {
    flashcard: Flashcard,
    flashcard_side: FlashcardSide,
}

/// Which flashcard side are we looking at?
#[derive(Default)]
enum FlashcardSide {
    #[default]
    Front,
    Back,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks to go back a screen                     
    Back,
    /// Show the user a toast
    AddToast(OboeteToast),
    /// Update the current folder id, needed for some database operations
    UpdateCurrentFolderId(i32),

    /// Load the flashcards into state
    LoadFlashcards,
    /// Callback after asking to load the flashcards into state
    FlashcardsLoaded(Result<(Vec<Flashcard>, f32), anywho::Error>),

    /// Ask to swap the currently studying flashcard side
    SwapFlashcardSide,
    /// Update the currently styuding flashcard status
    UpdateFlashcardStatus(i32, FlashcardStatus),
    /// Callback after updating the currently studying flashcard, select a new card...
    FlashcardStatusUpdated,
}

/// Allows us to talk with the parent screen
pub enum Action {
    None,
    Back(i32),
    Run(Task<Message>),
    AddToast(OboeteToast),
}

impl StudyScreen {
    /// Init the screen
    pub fn new(database: &Arc<Pool<Sqlite>>, folder_id: i32) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(
                Flashcard::get_all_with_retention_rate(Arc::clone(database), folder_id),
                Message::FlashcardsLoaded,
            )
            .chain(Task::done(Message::UpdateCurrentFolderId(folder_id))),
        )
    }

    /// View of the screen
    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => container(text("Loading...")).center(Length::Fill).into(),
            State::Ready {
                studying_flashcard,
                flashcards,
                current_index,
                current_mode,
                ..
            } => {
                let spacing = theme::active().cosmic().spacing;

                let content =
                    study_view(studying_flashcard, current_mode, flashcards, current_index);
                let buttons = study_buttons_view(spacing, studying_flashcard);

                container(
                    column![content, buttons]
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .spacing(spacing.space_s),
                )
                .padding(15)
                .center(Length::Fill)
                .into()
            }
        }
    }

    /// Handles interactions for this screen
    pub fn update(&mut self, message: Message, database: &Arc<Pool<Sqlite>>) -> Action {
        match message {
            Message::Back => {
                let State::Ready {
                    current_folder_id, ..
                } = &self.state
                else {
                    return Action::None;
                };

                if let Some(folder_id) = current_folder_id {
                    return Action::Back(*folder_id);
                }
                Action::None
            }
            Message::AddToast(toast) => Action::AddToast(toast),
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
                            Flashcard::get_all_with_retention_rate(Arc::clone(database), folder_id),
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
                    Ok((mut flashcards, retention_rate)) => {
                        if !flashcards.is_empty() {
                            if let Some((due_cards, current_mode)) =
                                order_due_cards(&mut flashcards)
                            {
                                let flashcard = due_cards.first().unwrap().to_owned();

                                self.state = State::Ready {
                                    scheduler: Box::from(
                                        FSRSScheduler::new(retention_rate).unwrap(),
                                    ),
                                    current_folder_id: None,
                                    flashcards: due_cards,
                                    studying_flashcard: StudyingFlashcard {
                                        flashcard,
                                        flashcard_side: FlashcardSide::default(),
                                    },
                                    current_index: 0,
                                    current_mode,
                                };
                            } else {
                                return self.update(Message::Back, &Arc::clone(database));
                            }
                        } else {
                            return self.update(Message::Back, &Arc::clone(database));
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        return Action::AddToast(OboeteToast::new(e));
                    }
                }
                Action::None
            }
            Message::SwapFlashcardSide => {
                let State::Ready {
                    studying_flashcard, ..
                } = &mut self.state
                else {
                    return Action::None;
                };

                match &studying_flashcard.flashcard_side {
                    FlashcardSide::Front => studying_flashcard.flashcard_side = FlashcardSide::Back,
                    FlashcardSide::Back => studying_flashcard.flashcard_side = FlashcardSide::Front,
                }

                Action::None
            }
            Message::UpdateFlashcardStatus(flashcard_id, flashcard_status) => {
                let State::Ready {
                    studying_flashcard,
                    scheduler,
                    ..
                } = &self.state
                else {
                    return Action::None;
                };

                let (new_memory_state, new_due_date) = match utils::update_fsrs_data(
                    &flashcard_status,
                    &studying_flashcard.flashcard,
                    scheduler,
                ) {
                    Some(data) => data,
                    None => {
                        eprintln!("Error updating FSRS Data");
                        return Action::Run(Task::done(Message::LoadFlashcards));
                    }
                };

                Action::Run(Task::perform(
                    Flashcard::update_status(
                        Arc::clone(database),
                        flashcard_status,
                        flashcard_id,
                        new_memory_state.into(),
                        new_due_date,
                    ),
                    |res| match res {
                        Ok(_) => Message::FlashcardStatusUpdated,
                        Err(e) => {
                            eprintln!("{}", e);
                            Message::AddToast(OboeteToast::new(e))
                        }
                    },
                ))
            }
            Message::FlashcardStatusUpdated => {
                let State::Ready {
                    flashcards,
                    current_index,
                    studying_flashcard,
                    ..
                } = &mut self.state
                else {
                    return Action::None;
                };

                // Move to next card
                let next_index = *current_index + 1;

                if next_index >= flashcards.len() {
                    // No more cards to study, go back
                    return self.update(Message::Back, database);
                }

                // Update to next card
                *current_index = next_index;
                let next_flashcard = flashcards[next_index].clone();

                *studying_flashcard = StudyingFlashcard {
                    flashcard: next_flashcard,
                    flashcard_side: FlashcardSide::default(),
                };

                Action::None
            }
        }
    }

    /// Subscriptions of this screen
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

/// View of the study page content
fn study_view<'a>(
    studying_flashcard: &'a StudyingFlashcard,
    practice_mode: &'a PracticeMode,
    flashcards: &'a [Flashcard],
    current_index: &usize,
) -> Element<'a, Message> {
    let flashcard_content: Element<Message> = match studying_flashcard.flashcard_side {
        FlashcardSide::Front => match &studying_flashcard.flashcard.front {
            FlashcardField::Text(t) => container(text(t).size(75)).center(Length::Fill).into(),
            FlashcardField::Image { path, alt_text } => container(
                container(tooltip(
                    container(image(path).content_fit(ContentFit::Contain)).padding(15),
                    container(text(alt_text).size(15))
                        .center(Length::Shrink)
                        .padding(5),
                    tooltip::Position::FollowCursor,
                ))
                .center(Length::Fill)
                .padding(20),
            )
            .center(Length::Fill)
            .into(),
        },
        FlashcardSide::Back => match &studying_flashcard.flashcard.back {
            FlashcardField::Text(t) => container(text(t).size(75)).center(Length::Fill).into(),
            FlashcardField::Image { path, alt_text } => container(
                container(tooltip(
                    container(image(path).content_fit(ContentFit::Contain)).padding(15),
                    container(text(alt_text).size(15))
                        .center(Length::Shrink)
                        .padding(5),
                    tooltip::Position::FollowCursor,
                ))
                .center(Length::Fill)
                .padding(20),
            )
            .center(Length::Fill)
            .into(),
        },
    };

    let mode_text = match practice_mode {
        PracticeMode::Fsrs => {
            fl!(
                "fsrs-mode",
                due = ((current_index + 1) as i64),
                total = flashcards.len()
            )
        }

        PracticeMode::Study => {
            fl!(
                "study-mode",
                number = ((current_index + 1) as i64),
                total = flashcards.len()
            )
        }
    };

    container(stack![
        container(mouse_area(flashcard_content).on_press(Message::SwapFlashcardSide))
            .style(|theme| {
                let mut a = theme::style::Container::primary(theme.cosmic());
                a.border = cosmic::iced::Border {
                    color: studying_flashcard.flashcard.status.get_border_color(),
                    width: 0.0,
                    radius: theme.cosmic().corner_radii.radius_s.into(),
                };
                a.shadow = cosmic::iced_core::Shadow {
                    color: studying_flashcard.flashcard.status.get_border_color(),
                    offset: cosmic::iced::Vector::new(0.0, 0.0),
                    blur_radius: 16.0,
                };
                a
            })
            .center(Length::Fill),
        container(text(mode_text).font(Font {
            weight: cosmic::iced::font::Weight::Bold,
            ..Default::default()
        }))
        .padding(10)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Top)
        .width(Length::Fill)
        .height(Length::Fill)
    ])
    .into()
}

/// View of the buttons of the study page
fn study_buttons_view<'a>(
    spacing: Spacing,
    studying_flashcard: &'a StudyingFlashcard,
) -> Element<'a, Message> {
    row![
        button::custom(text(fl!("bad-status")).center())
            .on_press(Message::UpdateFlashcardStatus(
                studying_flashcard.flashcard.id.unwrap_or_default(),
                FlashcardStatus::Bad
            ))
            .class(button_style(FlashcardStatus::Bad))
            .height(Length::Fixed(60.))
            .width(Length::Fill),
        button::custom(text(fl!("ok-status")).center())
            .on_press(Message::UpdateFlashcardStatus(
                studying_flashcard.flashcard.id.unwrap_or_default(),
                FlashcardStatus::Ok
            ))
            .class(button_style(FlashcardStatus::Ok))
            .height(Length::Fixed(60.))
            .width(Length::Fill),
        button::custom(text(fl!("good-status")).center())
            .on_press(Message::UpdateFlashcardStatus(
                studying_flashcard.flashcard.id.unwrap_or_default(),
                FlashcardStatus::Great
            ))
            .class(button_style(FlashcardStatus::Great))
            .height(Length::Fixed(60.))
            .width(Length::Fill),
        button::custom(text(fl!("easy-status")).center())
            .on_press(Message::UpdateFlashcardStatus(
                studying_flashcard.flashcard.id.unwrap_or_default(),
                FlashcardStatus::Easy
            ))
            .class(button_style(FlashcardStatus::Easy))
            .height(Length::Fixed(60.))
            .width(Length::Fill)
    ]
    .spacing(spacing.space_s)
    .into()
}

//
// HELPERS
//

/// Given the current [`FlashcardStatus`] gives back the appropiate [`theme::Button`]
fn button_style(status: FlashcardStatus) -> theme::Button {
    theme::Button::Custom {
        active: Box::new(move |_focused, theme| button_appearance(theme, status)),
        disabled: Box::new(move |theme| button_appearance(theme, status)),
        hovered: Box::new(move |_focused, theme| button_appearance(theme, status)),
        pressed: Box::new(move |_focused, theme| button_appearance(theme, status)),
    }
}

/// Helper for giving the appropiate [`theme::Button`] for a given [`FlashcardStatus`]
fn button_appearance(theme: &theme::Theme, status: FlashcardStatus) -> button::Style {
    let cosmic = theme.cosmic();
    let mut appearance = button::Style::new();

    appearance.border_radius = cosmic.radius_xs().into();
    appearance.icon_color = Some(Color::from(cosmic.on_accent_color()));
    appearance.text_color = Some(Color::from(cosmic.on_accent_color()));
    appearance.outline_width = 1.0;
    appearance.border_width = 2.0;

    appearance.background = Some(status.get_color().into());
    appearance.border_color = status.get_border_color();

    appearance
}

/// Orders the [`Flashcard`] to follow the FSRS algo if possible, if not offers ALL cards sorted by due date, also determines the page [`PracticeMode`]
fn order_due_cards(flashcards: &mut [Flashcard]) -> Option<(Vec<Flashcard>, PracticeMode)> {
    let current_day = utils::current_day();

    // Separate due and not-due cards
    let mut due_cards: Vec<Flashcard> = flashcards
        .iter()
        .filter(|card| card.is_due())
        .cloned()
        .collect();

    // Sort due cards by due date (study overdue cards first)
    due_cards.sort_by_key(|card| card.due_date.unwrap_or(current_day));

    // If no cards are due, offer ALL cards sorted by due date (earliest first)
    if due_cards.is_empty() && !flashcards.is_empty() {
        let mut all_cards = flashcards.to_vec();
        all_cards.sort_by_key(|card| card.due_date.unwrap_or(current_day));
        return Some((all_cards, PracticeMode::Study));
    }

    if due_cards.is_empty() {
        None
    } else {
        Some((due_cards, PracticeMode::Fsrs))
    }
}
