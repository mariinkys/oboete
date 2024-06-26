use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Length, Padding,
    },
    theme, widget, Element,
};

use crate::{fl, models::Flashcard, utils::select_random_flashcard};

const FLASHCARDS_PER_ROW: usize = 5;

pub struct Flashcards {
    pub current_folder_id: i32,
    pub flashcards: Vec<Flashcard>,
    pub new_edit_flashcard: CreateEditFlashcardState,
}

pub struct CreateEditFlashcardState {
    id: Option<i32>,
    front: String,
    back: String,
    status: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Upsert,
    Upserted,
    LoadedSingle(Flashcard),
    SetFlashcards(Vec<Flashcard>),
    ToggleCreatePage(Option<Flashcard>),
    StudyFlashcards,
    ContextPageFrontInput(String),
    ContextPageBackInput(String),
}

pub enum Command {
    //The i32 is the Folder Id
    LoadFlashcards(i32),
    ToggleCreateFlashcardPage(Option<Flashcard>),
    UpsertFlashcard(Flashcard),
    OpenStudyFolderFlashcardsPage,
}

impl Flashcards {
    pub fn new() -> Self {
        Self {
            current_folder_id: 0,
            flashcards: Vec::new(),
            new_edit_flashcard: CreateEditFlashcardState {
                id: None,
                front: String::new(),
                back: String::new(),
                status: 0,
            },
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
                self.new_edit_flashcard = CreateEditFlashcardState {
                    id: None,
                    front: String::new(),
                    back: String::new(),
                    status: 0,
                };
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
                    self.new_edit_flashcard = CreateEditFlashcardState {
                        id: None,
                        front: String::new(),
                        back: String::new(),
                        status: 0,
                    };
                }

                commands.push(Command::ToggleCreateFlashcardPage(flashcard))
            }
            Message::StudyFlashcards => commands.push(Command::OpenStudyFolderFlashcardsPage),
            Message::ContextPageFrontInput(value) => self.new_edit_flashcard.front = value,
            Message::ContextPageBackInput(value) => self.new_edit_flashcard.back = value,
        }

        commands
    }

    fn flashcard_header_row(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let new_flashcard_button = widget::button(widget::text("New"))
            .style(theme::Button::Suggested)
            .padding(spacing.space_xxs)
            .on_press(Message::ToggleCreatePage(None));

        let study_button = widget::button(widget::text("Study"))
            .style(theme::Button::Suggested)
            .padding(spacing.space_xxs)
            .on_press(Message::StudyFlashcards);

        widget::row::with_capacity(3)
            .align_items(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text::title3("Flashcards").width(Length::Fill)) //TODO: The Title should be the Folder name
            .push(study_button)
            .push(new_flashcard_button)
            .into()
    }

    pub fn view(&self) -> Element<Message> {
        //TODO: Fix design, what happens when a item has a name that is longer?...
        //All flashcards should have the same with ej: 25% 25% 25% 25%...
        let mut flashcards_grid = widget::Grid::new()
            .width(Length::Fill)
            .column_alignment(cosmic::iced::Alignment::Center);

        for (index, flashcard) in self.flashcards.iter().enumerate() {
            let flashcard_button = widget::button(
                widget::container::Container::new(widget::text(flashcard.front.as_str()))
                    .style(theme::Container::Card)
                    .padding(Padding::new(10.0)),
            )
            .on_press_down(Message::ToggleCreatePage(Some(flashcard.clone())))
            .style(theme::Button::Text)
            .width(Length::Shrink);

            if index % FLASHCARDS_PER_ROW == 0 {
                flashcards_grid = flashcards_grid.insert_row();
            }

            flashcards_grid = flashcards_grid.push(flashcard_button);
        }

        widget::Column::new()
            .push(self.flashcard_header_row())
            .push(flashcards_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// The create or edit flashcard context page for this app.
    pub fn create_edit_flashcard_contextpage(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        widget::settings::view_column(vec![widget::settings::view_section(fl!(
            "flashcard-details"
        ))
        .add(
            widget::column::with_children(vec![
                widget::text::body(fl!("new-flashcard-front-title")).into(),
                widget::text_input(
                    fl!("new-flashcard-front-inputfield"),
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
                widget::text::body(fl!("new-flashcard-back-title")).into(),
                widget::text_input(
                    fl!("new-flashcard-back-inputfield"),
                    &self.new_edit_flashcard.back,
                )
                .on_input(Message::ContextPageBackInput)
                .into(),
            ])
            .spacing(spacing.space_xxs)
            .padding([0, 15, 0, 15]),
        )
        .add(match self.new_edit_flashcard.id {
            Some(_id) => widget::button(
                widget::text(fl!("new-flashcard-edit-button"))
                    .horizontal_alignment(cosmic::iced::alignment::Horizontal::Center)
                    .width(Length::Fill),
            )
            .on_press(Message::Upsert)
            .style(theme::Button::Suggested)
            .padding([10, 0, 10, 0])
            .width(Length::Fill),
            None => widget::button(
                widget::text(fl!("new-flashcard-submit-button"))
                    .horizontal_alignment(cosmic::iced::alignment::Horizontal::Center)
                    .width(Length::Fill),
            )
            .on_press(Message::Upsert)
            .style(theme::Button::Suggested)
            .padding([10, 0, 10, 0])
            .width(Length::Fill),
        })
        .into()])
        .into()
    }

    pub fn view_study_page(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        //TODO: I can't do this here, every time I click anywhere it updates
        let flashcard = select_random_flashcard(&self.flashcards).unwrap_or(Flashcard {
            id: None,
            front: String::from("Error"),
            back: String::from("Error"),
            status: 0,
        });

        let flashcard_container = widget::container(
            widget::button(
                widget::Text::new(flashcard.front)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .style(theme::Button::Text)
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .style(theme::Container::ContextDrawer)
        .width(Length::Fill)
        .height(Length::Fill);

        //TODO: Custom Button Styling
        let options_row = widget::row::with_capacity(3)
            .push(
                widget::button(
                    widget::Text::new("Bad")
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .style(theme::Button::Suggested)
                .height(Length::Fixed(60.0))
                .width(Length::Fill),
            )
            .push(
                widget::button(
                    widget::Text::new("Ok")
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .style(theme::Button::Suggested)
                .height(Length::Fixed(60.0))
                .width(Length::Fill),
            )
            .push(
                widget::button(
                    widget::Text::new("Good")
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .style(theme::Button::Suggested)
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
}
