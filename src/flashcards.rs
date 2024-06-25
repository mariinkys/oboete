use cosmic::{
    iced::{Length, Padding},
    theme, widget, Element,
};

use crate::{fl, models::Flashcard};

const FLASHCARDS_PER_ROW: usize = 5;

pub struct Flashcards {
    pub current_folder_id: i32,
    pub flashcards: Vec<Flashcard>,
    pub new_flashcard: NewFlashcardState,
}

pub struct NewFlashcardState {
    front: String,
    back: String,
    status: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Create,
    Created,
    SetFlashcards(Vec<Flashcard>),
    ToggleCreatePage,
    NewFlashcardFrontInput(String),
    NewFlashcardBackInput(String),
}

pub enum Command {
    //The i32 is the Folder Id
    LoadFlashcards(i32),
    ToggleCreateFlashcardPage,
    CreateFlashcard(Flashcard),
    //The i32 is the Flashcard Id
    OpenFlashcard(i32),
}

impl Flashcards {
    pub fn new() -> Self {
        Self {
            current_folder_id: 0,
            flashcards: Vec::new(),
            new_flashcard: NewFlashcardState {
                front: String::new(),
                back: String::new(),
                status: 0,
            },
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = Vec::new();

        match message {
            Message::Create => commands.push(Command::CreateFlashcard(Flashcard {
                id: None,
                front: self.new_flashcard.front.to_string(),
                back: self.new_flashcard.back.to_string(),
                status: 0,
            })),
            Message::SetFlashcards(flashcards) => self.flashcards = flashcards,
            Message::ToggleCreatePage => commands.push(Command::ToggleCreateFlashcardPage),
            Message::NewFlashcardFrontInput(value) => self.new_flashcard.front = value,
            Message::NewFlashcardBackInput(value) => self.new_flashcard.back = value,
            Message::Created => {
                self.new_flashcard = NewFlashcardState {
                    front: String::new(),
                    back: String::new(),
                    status: 0,
                };
                commands.push(Command::LoadFlashcards(self.current_folder_id))
            }
        }

        commands
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
            .on_press_down(Message::ToggleCreatePage) //TODO: This should open the flashcard to view/edit, this is just for testing
            .style(theme::Button::Text)
            .width(Length::Shrink);

            if index % FLASHCARDS_PER_ROW == 0 {
                flashcards_grid = flashcards_grid.insert_row();
            }

            flashcards_grid = flashcards_grid.push(flashcard_button);
        }

        let new_flashcard_button = widget::button(widget::text("New"))
            .style(theme::Button::Suggested)
            .on_press(Message::ToggleCreatePage);

        let header_row = widget::Row::new()
            .push(new_flashcard_button)
            .width(Length::Fill);

        widget::Column::new()
            .push(header_row)
            .push(flashcards_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// The new flashcard context page for this app.
    pub fn new_flashcard_contextpage(&self) -> Element<Message> {
        let new_flashcard_front_inputfield = widget::TextInput::new(
            fl!("new-flashcard-front-inputfield"),
            &self.new_flashcard.front,
        )
        .on_input(Message::NewFlashcardFrontInput);

        let new_flashcard_back_inputfield = widget::TextInput::new(
            fl!("new-flashcard-back-inputfield"),
            &self.new_flashcard.back,
        )
        .on_input(Message::NewFlashcardBackInput);

        let submit_button = widget::button(widget::text(fl!("new-flashcard-submit-button")))
            .on_press(Message::Create)
            .style(theme::Button::Suggested);

        widget::Column::new()
            .push(new_flashcard_front_inputfield)
            .push(new_flashcard_back_inputfield)
            .push(submit_button)
            .width(Length::Fill)
            .into()
    }
}
