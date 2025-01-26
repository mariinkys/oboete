// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Color, Length,
    },
    theme,
    widget::{self},
    Element,
};

use crate::{
    fl,
    oboete::{models::flashcard::Flashcard, utils::select_random_flashcard},
};

pub struct StudyPage {
    flashcards: Vec<Flashcard>,
    currently_studying_flashcard: Flashcard,
    currently_studying_flashcard_side: CurrentFlashcardSide,
}

#[derive(Debug, Clone)]
pub enum CurrentFlashcardSide {
    Front,
    Back,
}

#[derive(Debug, Clone)]
pub enum StudyActions {
    Bad,
    Ok,
    Good,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetFlashcards(Vec<Flashcard>),

    SwapFlashcardSide,
    UpdateFlashcardStatus(Flashcard, StudyActions),

    UpdatedFlashcardStatus(Vec<Flashcard>),
}

pub enum StudyPageTask {
    UpdateFlashcardStatus(Flashcard),
}

impl StudyPage {
    pub fn init() -> Self {
        Self {
            flashcards: Vec::new(),
            currently_studying_flashcard: Flashcard::new_error_variant(),
            currently_studying_flashcard_side: CurrentFlashcardSide::Front,
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<StudyPageTask> {
        let mut tasks = Vec::new();

        match message {
            // Sets the given flashcards to the appstate and selects a random flashcard
            Message::SetFlashcards(flashcards) => {
                self.flashcards = flashcards;
                self.currently_studying_flashcard = select_random_flashcard(&self.flashcards)
                    .unwrap_or(Flashcard::new_error_variant());
            }

            // Flips the current studying flashcard side
            Message::SwapFlashcardSide => match self.currently_studying_flashcard_side {
                CurrentFlashcardSide::Front => {
                    self.currently_studying_flashcard_side = CurrentFlashcardSide::Back
                }
                CurrentFlashcardSide::Back => {
                    self.currently_studying_flashcard_side = CurrentFlashcardSide::Front
                }
            },

            // Updates the local flashcard status and asks for it to be updated on the db
            Message::UpdateFlashcardStatus(mut flashcard, action) => {
                match action {
                    StudyActions::Bad => flashcard.status = 1,
                    StudyActions::Ok => flashcard.status = 2,
                    StudyActions::Good => flashcard.status = 3,
                }

                tasks.push(StudyPageTask::UpdateFlashcardStatus(flashcard))
            }

            // Callback after status update
            Message::UpdatedFlashcardStatus(flashcards) => {
                self.flashcards = flashcards;
                self.currently_studying_flashcard_side = CurrentFlashcardSide::Front;
                self.currently_studying_flashcard = select_random_flashcard(&self.flashcards)
                    .unwrap_or(Flashcard::new_error_variant());
            }
        }

        tasks
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let flashcard_container = widget::container(
            widget::button::custom(
                // Text depends on if we are looking at the front or the back
                widget::Text::new(match self.currently_studying_flashcard_side {
                    CurrentFlashcardSide::Front => &self.currently_studying_flashcard.front,
                    CurrentFlashcardSide::Back => &self.currently_studying_flashcard.back,
                })
                .size(spacing.space_xxl)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(Vertical::Center)
                .align_x(Horizontal::Center),
            )
            .on_press(Message::SwapFlashcardSide)
            .class(button_style(false, false, ButtonStyle::NoHover))
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .class(container_appearance(
            self.currently_studying_flashcard.status,
        ))
        .width(Length::Fill)
        .height(Length::Fill);

        let options_row = widget::row::with_capacity(3)
            .push(
                widget::button::custom(
                    widget::Text::new(fl!("bad-status"))
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .on_press(Message::UpdateFlashcardStatus(
                    self.currently_studying_flashcard.clone(),
                    StudyActions::Bad,
                ))
                .class(button_style(false, false, ButtonStyle::BadButton))
                .height(Length::Fixed(60.0))
                .width(Length::Fill),
            )
            .push(
                widget::button::custom(
                    widget::Text::new(fl!("ok-status"))
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .on_press(Message::UpdateFlashcardStatus(
                    self.currently_studying_flashcard.clone(),
                    StudyActions::Ok,
                ))
                .class(button_style(false, false, ButtonStyle::OkButton))
                .height(Length::Fixed(60.0))
                .width(Length::Fill),
            )
            .push(
                widget::button::custom(
                    widget::Text::new(fl!("good-status"))
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .on_press(Message::UpdateFlashcardStatus(
                    self.currently_studying_flashcard.clone(),
                    StudyActions::Good,
                ))
                .class(button_style(false, false, ButtonStyle::GoodButton))
                .height(Length::Fixed(60.0))
                .width(Length::Fill),
            )
            .align_y(cosmic::iced::Alignment::Center)
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ButtonStyle {
    NoHover, // For the front/back container button
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
) -> widget::button::Style {
    let cosmic = theme.cosmic();
    let mut appearance = widget::button::Style::new();

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
            r: 191.0 / 255.0,
            g: 57.0 / 255.0,
            b: 57.0 / 255.0,
            a: 0.75,
        },
        ButtonStyle::NoHover => Color::from(cosmic.bg_color()),
    };

    appearance.background = Some(custom_bg_color.into());
    appearance.border_color = custom_border_color;

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

fn container_appearance<'a>(flashard_status: i32) -> theme::Container<'a> {
    // Got this from: https://github.com/pop-os/cosmic-files/blob/master/src/tab.rs#L2038
    // New reference: https://github.com/pop-os/libcosmic/blob/master/src/theme/style/iced.rs
    theme::Container::custom(move |t| {
        let mut a = theme::style::Container::primary(t.cosmic());

        let custom_border_color = match flashard_status {
            // Orange (Ok Flashcard)
            2 => Color {
                r: 245.0 / 255.0,
                g: 188.0 / 255.0,
                b: 66.0 / 255.0,
                a: 0.75,
            },
            // Green (Good Flashcard)
            3 => Color {
                r: 21.0 / 255.0,
                g: 191.0 / 255.0,
                b: 89.0 / 255.0,
                a: 0.75,
            },
            // Red (Bad Flashcard)
            1 => Color {
                r: 107.0 / 255.0,
                g: 7.0 / 255.0,
                b: 7.0 / 255.0,
                a: 1.0,
            },
            _ => a.border.color,
        };

        a.border = cosmic::iced::Border {
            color: custom_border_color,
            width: 0.0,
            radius: t.cosmic().corner_radii.radius_s.into(),
        };
        a.shadow = cosmic::iced_core::Shadow {
            color: custom_border_color,
            offset: cosmic::iced::Vector::new(0.0, 0.0),
            blur_radius: 16.0,
        };
        a
    })
}
