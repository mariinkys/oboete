use cosmic::{
    iced::{Length, Padding},
    theme, widget, Element,
};

use crate::{fl, models::StudySet};

const STUDYSETS_PER_ROW: usize = 5;

pub struct StudySets {
    pub studysets: Vec<StudySet>,
    pub new_studyset: NewStudySetState,
}

pub struct NewStudySetState {
    name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Create,
    Created,
    GetStudySets,
    SetStudySets(Vec<StudySet>),
    ToggleCreatePage,
    NewStudySetNameInput(String),
}

pub enum Command {
    LoadStudySets,
    ToggleCreateStudySetPage,
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
            Message::GetStudySets => commands.push(Command::LoadStudySets),
            Message::Create => commands.push(Command::CreateStudySet(StudySet {
                id: None,
                name: self.new_studyset.name.to_string(),
                folders: Vec::new(),
            })),
            Message::SetStudySets(studysets) => self.studysets = studysets,
            Message::ToggleCreatePage => commands.push(Command::ToggleCreateStudySetPage),
            Message::NewStudySetNameInput(value) => self.new_studyset.name = value,
            Message::Created => {
                self.new_studyset = NewStudySetState {
                    name: String::new(),
                };
                commands.push(Command::LoadStudySets)
            }
        }

        commands
    }

    pub fn view(&self) -> Element<Message> {
        //TODO: Fix design, what happens when a item has a name that is longer?...
        //All studysets should have the same with ej: 25% 25% 25% 25%...
        let mut studysets_grid = widget::Grid::new()
            .width(Length::Fill)
            .column_alignment(cosmic::iced::Alignment::Center);

        for (index, studyset) in self.studysets.iter().enumerate() {
            let studyset_button = widget::button(
                widget::container::Container::new(widget::text(studyset.name.as_str()))
                    .style(theme::Container::Card)
                    .padding(Padding::new(10.0)),
            )
            .on_press_down(Message::ToggleCreatePage) //TODO: This should open the studyset, this is just for testing
            .style(theme::Button::Text)
            .width(Length::Shrink);

            if index % STUDYSETS_PER_ROW == 0 {
                studysets_grid = studysets_grid.insert_row();
            }

            studysets_grid = studysets_grid.push(studyset_button);
        }

        let new_studyset_button = widget::button(widget::text("New"))
            .style(theme::Button::Suggested)
            .on_press(Message::ToggleCreatePage);

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
