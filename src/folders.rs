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

        let new_folder_button = widget::button(widget::text("New"))
            .style(theme::Button::Suggested)
            .on_press(Message::ToggleCreatePage);

        let header_row = widget::Row::new()
            .push(new_folder_button)
            .width(Length::Fill);

        widget::Column::new()
            .push(header_row)
            .push(folders_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// The new folder context page for this app.
    pub fn new_folder_contextpage(&self) -> Element<Message> {
        let new_folder_name_inputfield =
            widget::TextInput::new(fl!("new-folder-name-inputfield"), &self.new_folder.name)
                .on_input(Message::NewFolderNameInput);

        let submit_button = widget::button(widget::text(fl!("new-folder-submit-button")))
            .on_press(Message::Create)
            .style(theme::Button::Suggested);

        widget::Column::new()
            .push(new_folder_name_inputfield)
            .push(submit_button)
            .width(Length::Fill)
            .into()
    }
}
