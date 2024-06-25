use cosmic::{
    iced::{Length, Padding},
    theme, widget, Element,
};

use crate::{fl, models::Folder};

const FOLDERS_PER_ROW: usize = 5;

pub struct Folders {
    pub current_studyset_id: i32,
    pub folders: Vec<Folder>,
    pub new_folder: NewFolderState,
}

pub struct NewFolderState {
    name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Create,
    Created,
    SetFolders(Vec<Folder>),
    ToggleCreatePage,
    NewFolderNameInput(String),
    OpenFolder(i32),
}

pub enum Command {
    //The i32 is the Studyset Id
    LoadFolders(i32),
    ToggleCreateFolderPage,
    CreateFolder(Folder),
    //The i32 is the Folder Id
    OpenFolder(i32),
}

impl Folders {
    pub fn new() -> Self {
        Self {
            current_studyset_id: 0,
            folders: Vec::new(),
            new_folder: NewFolderState {
                name: String::new(),
            },
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = Vec::new();

        match message {
            Message::Create => commands.push(Command::CreateFolder(Folder {
                id: None,
                name: self.new_folder.name.to_string(),
                flashcards: Vec::new(),
            })),
            Message::SetFolders(folders) => self.folders = folders,
            Message::ToggleCreatePage => commands.push(Command::ToggleCreateFolderPage),
            Message::NewFolderNameInput(value) => self.new_folder.name = value,
            Message::Created => {
                self.new_folder = NewFolderState {
                    name: String::new(),
                };
                commands.push(Command::LoadFolders(self.current_studyset_id))
            }
            Message::OpenFolder(id) => commands.push(Command::OpenFolder(id)),
        }

        commands
    }

    fn folder_header_row(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let new_folder_button = widget::button(widget::text("New"))
            .style(theme::Button::Suggested)
            .padding(spacing.space_xxs)
            .on_press(Message::ToggleCreatePage);

        widget::row::with_capacity(2)
            .align_items(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text::title3("Folders").width(Length::Fill)) //TODO: The Title should be the StudySet name
            .push(new_folder_button)
            .into()
    }

    pub fn view(&self) -> Element<Message> {
        //TODO: Fix design, what happens when a item has a name that is longer?...
        //All studysets should have the same with ej: 25% 25% 25% 25%...
        let mut folders_grid = widget::Grid::new()
            .width(Length::Fill)
            .column_alignment(cosmic::iced::Alignment::Center);

        for (index, folder) in self.folders.iter().enumerate() {
            let folder_button = widget::button(
                widget::container::Container::new(widget::text(folder.name.as_str()))
                    .style(theme::Container::Card)
                    .padding(Padding::new(10.0)),
            )
            .on_press_down(Message::OpenFolder(folder.id.unwrap()))
            .style(theme::Button::Text)
            .width(Length::Shrink);

            if index % FOLDERS_PER_ROW == 0 {
                folders_grid = folders_grid.insert_row();
            }

            folders_grid = folders_grid.push(folder_button);
        }

        widget::Column::new()
            .push(self.folder_header_row())
            .push(folders_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// The new folder context page for this app.
    pub fn new_folder_contextpage(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        widget::settings::view_column(vec![widget::settings::view_section(fl!("new-folder"))
            .add(
                widget::column::with_children(vec![
                    widget::text::body(fl!("new-folder-name-title")).into(),
                    widget::text_input(fl!("new-folder-name-inputfield"), &self.new_folder.name)
                        .on_input(Message::NewFolderNameInput)
                        .into(),
                ])
                .spacing(spacing.space_xxs)
                .padding([0, 15, 0, 15]),
            )
            .add(
                widget::button(
                    widget::text(fl!("new-folder-submit-button"))
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
