use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length,
    },
    theme, widget, Apply, Element,
};

use crate::{fl, models::Folder};

pub struct Folders {
    pub current_studyset_id: Option<i32>,
    pub folders: Vec<Folder>,
    pub new_folder: NewFolderState,
}

pub struct NewFolderState {
    id: Option<i32>,
    name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenCreateFolderDialog,
    Upsert,
    Upserted,
    LoadedSingle(Folder),
    Load(Option<i32>),
    SetFolders(Vec<Folder>),
    NewFolderNameInput(String),
    OpenFolder(i32),
    ToggleEditContextPage(Option<Folder>),
}

pub enum Command {
    //The i32 is the Studyset Id
    LoadFolders(i32),
    //The i32 is the Folder Id
    OpenFolder(i32),
    UpsertFolder(Folder),
    OpenCreateFolderDialog,
    ToggleEditContextPage(Option<Folder>),
}

impl Folders {
    pub fn new() -> Self {
        Self {
            current_studyset_id: None,
            folders: Vec::new(),
            new_folder: NewFolderState {
                id: None,
                name: String::new(),
            },
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = Vec::new();

        match message {
            Message::OpenCreateFolderDialog => commands.push(Command::OpenCreateFolderDialog),
            Message::Upsert => commands.push(Command::UpsertFolder(Folder {
                id: self.new_folder.id,
                name: self.new_folder.name.to_string(),
                flashcards: Vec::new(),
            })),
            Message::Upserted => {
                self.new_folder = NewFolderState {
                    id: None,
                    name: String::new(),
                };
                commands.push(Command::LoadFolders(self.current_studyset_id.unwrap()))
            }
            Message::LoadedSingle(folder) => {
                self.new_folder = NewFolderState {
                    id: folder.id,
                    name: folder.name,
                };
            }
            Message::Load(studyset_id) => match studyset_id {
                Some(set_id) => commands.push(Command::LoadFolders(set_id)),
                None => self.current_studyset_id = None,
            },
            Message::SetFolders(folders) => self.folders = folders,
            Message::NewFolderNameInput(value) => self.new_folder.name = value,
            Message::OpenFolder(id) => commands.push(Command::OpenFolder(id)),
            Message::ToggleEditContextPage(folder) => {
                if folder.is_none() {
                    self.new_folder = NewFolderState {
                        id: None,
                        name: String::new(),
                    };
                }

                commands.push(Command::ToggleEditContextPage(folder))
            }
        }
        commands
    }

    fn folder_header_row(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let new_folder_button = widget::button(widget::text("New"))
            .style(theme::Button::Suggested)
            .padding(spacing.space_xxs)
            .on_press(Message::OpenCreateFolderDialog);

        widget::row::with_capacity(2)
            .align_items(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text::title3("Folders").width(Length::Fill)) //TODO: The Title should be the StudySet name
            .push(new_folder_button)
            .into()
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        //TODO: Folders should have icons instead of text
        if self.current_studyset_id.is_some() {
            let mut folders = widget::list::list_column()
                .style(theme::Container::ContextDrawer)
                .spacing(spacing.space_xxxs)
                .padding([spacing.space_none, spacing.space_xxs]);

            for folder in &self.folders {
                let edit_button = widget::button(widget::text("Edit"))
                    .padding(spacing.space_xxs)
                    .style(theme::Button::Standard)
                    .on_press(Message::ToggleEditContextPage(Some(folder.clone())));

                let open_button = widget::button(widget::text("Open"))
                    .padding(spacing.space_xxs)
                    .style(theme::Button::Suggested)
                    .width(Length::Shrink)
                    .on_press(Message::OpenFolder(folder.id.unwrap()));

                let delete_button = widget::button("Delete")
                    .padding(spacing.space_xxs)
                    .style(theme::Button::Destructive);
                //.on_press(Message::DeleteFolder(id));

                let folder_name = widget::text(folder.name.clone())
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Left)
                    .width(Length::Fill);

                let row = widget::row::with_capacity(2)
                    .align_items(Alignment::Center)
                    .spacing(spacing.space_xxs)
                    .padding([spacing.space_xxxs, spacing.space_xxs])
                    .push(open_button)
                    .push(folder_name)
                    .push(delete_button)
                    .push(edit_button);

                folders = folders.add(row);
            }

            widget::column::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(self.folder_header_row())
                .push(folders)
                .apply(widget::container)
                .height(Length::Shrink)
                .apply(widget::scrollable)
                .height(Length::Fill)
                .into()
        } else {
            widget::Container::new(widget::Text::new("Empty").size(spacing.space_xl))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .align_y(cosmic::iced::alignment::Vertical::Center)
                .into()
        }
    }

    /// The edit folder context page for this app.
    pub fn edit_folder_contextpage(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        widget::settings::view_column(vec![widget::settings::view_section(fl!("folder-details"))
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
                    widget::text(fl!("new-folder-edit-button"))
                        .horizontal_alignment(cosmic::iced::alignment::Horizontal::Center)
                        .width(Length::Fill),
                )
                .on_press(Message::Upsert)
                .style(theme::Button::Suggested)
                .padding([10, 0, 10, 0])
                .width(Length::Fill),
            )
            .into()])
        .into()
    }
}
