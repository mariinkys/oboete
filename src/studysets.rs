use cosmic::{iced::Length, theme, widget, Element};

use crate::{fl, models::StudySet, utils::OboeteError};

const STUDYSETS_PER_ROW: usize = 5;

pub struct StudySets {
    studysets: Vec<StudySet>,
    new_studyset: NewStudySetState,
}

pub struct NewStudySetState {
    name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Create,
    StudySetsLoaded(Result<Vec<StudySet>, OboeteError>),
    StudySetCreated,
    NewStudySetNameInput(String),
}

pub enum Command {
    CreateStudySet(StudySet),
}

impl StudySets {
    pub fn new() -> Self {
        Self {
            studysets: Vec::new(),
            new_studyset: NewStudySetState {
                name: String::new(),
            },
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = Vec::new();

        match message {
            Message::Create => commands.push(Command::CreateStudySet(StudySet {
                id: None,
                name: self.new_studyset.name.to_string(),
                folders: Vec::new(),
            })),
            Message::StudySetsLoaded(studysets) => match studysets {
                Ok(value) => self.studysets = value,
                Err(_) => self.studysets = Vec::new(),
            },
            Message::NewStudySetNameInput(value) => self.new_studyset.name = value,
            Message::StudySetCreated => todo!(),
        }

        commands
    }

    pub fn view(&self) -> Element<Message> {
        let mut studysets_grid = widget::Grid::new().width(Length::Fill);

        for (index, studyset) in self.studysets.iter().enumerate() {
            let studyset_button =
                widget::button(widget::text(studyset.name.as_str())).style(theme::Button::Text);

            if index % STUDYSETS_PER_ROW == 0 {
                studysets_grid = studysets_grid.insert_row();
            }

            studysets_grid = studysets_grid.push(studyset_button);
        }

        let new_studyset_button =
            widget::button(widget::text("New")).style(theme::Button::Suggested);
        //.on_press(Message::ToggleContextPage(ContextPage::NewStudySet));

        let header_row = widget::Row::new()
            .push(new_studyset_button)
            .width(Length::Fill);

        widget::Column::new()
            .push(header_row)
            .push(studysets_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// The new studyset context page for this app.
    pub fn new_studyset_contextpage(&self) -> Element<Message> {
        let new_studyset_name_inputfield =
            widget::TextInput::new(fl!("new-studyset-name-inputfield"), &self.new_studyset.name)
                .on_input(Message::NewStudySetNameInput);

        let submit_button = widget::button(widget::text(fl!("new-studyset-submit-button")))
            .on_press(Message::Create)
            .style(theme::Button::Suggested);

        widget::Column::new()
            .push(new_studyset_name_inputfield)
            .push(submit_button)
            .width(Length::Fill)
            .into()
    }
}
