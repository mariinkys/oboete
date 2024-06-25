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
    OpenStudySet(i32),
}

pub enum Command {
    LoadStudySets,
    ToggleCreateStudySetPage,
    CreateStudySet(StudySet),
    //The i32 is the StudySetId
    OpenStudySet(i32),
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
            Message::OpenStudySet(id) => commands.push(Command::OpenStudySet(id)),
        }

        commands
    }

    fn studysets_header_row(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let new_studyset_button = widget::button(widget::text("New"))
            .style(theme::Button::Suggested)
            .padding(spacing.space_xxs)
            .on_press(Message::ToggleCreatePage);

        widget::row::with_capacity(2)
            .align_items(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text::title3("Study Sets").width(Length::Fill))
            .push(new_studyset_button)
            .into()
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
            .on_press_down(Message::OpenStudySet(studyset.id.unwrap()))
            .style(theme::Button::Text)
            .width(Length::Shrink);

            if index % STUDYSETS_PER_ROW == 0 {
                studysets_grid = studysets_grid.insert_row();
            }

            studysets_grid = studysets_grid.push(studyset_button);
        }

        widget::Column::new()
            .push(self.studysets_header_row())
            .push(studysets_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// The new studyset context page for this app.
    pub fn new_studyset_contextpage(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        widget::settings::view_column(vec![widget::settings::view_section(fl!("new-studyset"))
            .add(
                widget::column::with_children(vec![
                    widget::text::body(fl!("new-studyset-name-title")).into(),
                    widget::text_input(
                        fl!("new-studyset-name-inputfield"),
                        &self.new_studyset.name,
                    )
                    .on_input(Message::NewStudySetNameInput)
                    .into(),
                ])
                .spacing(spacing.space_xxs)
                .padding([0, 15, 0, 15]),
            )
            .add(
                widget::button(
                    widget::text(fl!("new-studyset-submit-button"))
                        .horizontal_alignment(cosmic::iced::alignment::Horizontal::Center)
                        .width(Length::Fill),
                )
                .on_press(Message::Create)
                .style(theme::Button::Suggested)
                .padding([10, 0, 10, 0])
                .width(Length::Fill),
            )
            .into()])
        .into()
    }
}
