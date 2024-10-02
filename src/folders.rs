use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length,
    },
    theme, widget, Apply, Element,
};

use crate::{core::icon_cache::IconCache, fl, models::Folder};

pub struct Folders {
    pub current_studyset_id: Option<i32>,
    pub folders: Vec<Folder>,
    pub new_folder: NewFolderState,
}

pub struct NewFolderState {
    id: Option<i32>,
    name: String,
}

impl NewFolderState {
    pub fn new() -> NewFolderState {
        NewFolderState {
            id: None,
            name: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenCreateFolderDialog,
    ToggleEditContextPage(Option<Folder>),

    LoadFolders,
    Upsert,
    Delete(Option<i32>),

    Upserted,
    SetFolders(Vec<Folder>),
    LoadedSingle(Folder),
    OpenFolder(i32),
    NewFolderNameInput(String),
}

pub enum Command {
    //The i32 is the Studyset Id
    LoadFolders(i32),
    //The i32 is the Folder Id
    OpenFolder(i32),
    UpsertFolder(Folder),
    OpenCreateFolderDialog,
    ToggleEditContextPage(Option<Folder>),
    DeleteFolder(Option<i32>),
}

impl Folders {
    pub fn new() -> Self {
        Self {
            current_studyset_id: None,
            folders: Vec::new(),
            new_folder: NewFolderState::new(),
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
                self.new_folder = NewFolderState::new();
                commands.push(Command::LoadFolders(self.current_studyset_id.unwrap()))
            }
            Message::LoadedSingle(folder) => {
                self.new_folder = NewFolderState {
                    id: folder.id,
                    name: folder.name,
                };
            }
            Message::LoadFolders => match self.current_studyset_id {
                Some(set_id) => commands.push(Command::LoadFolders(set_id)),
                None => self.current_studyset_id = None,
            },
            Message::SetFolders(folders) => self.folders = folders,
            Message::NewFolderNameInput(value) => self.new_folder.name = value,
            Message::OpenFolder(id) => commands.push(Command::OpenFolder(id)),
            Message::ToggleEditContextPage(folder) => {
                if folder.is_none() {
                    self.new_folder = NewFolderState::new();
                }

                commands.push(Command::ToggleEditContextPage(folder))
            }
            Message::Delete(folder_id) => commands.push(Command::DeleteFolder(folder_id)),
        }
        commands
    }

    fn folder_header_row(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        //TODO: IconCache::get("add-symbolic", 18) - For now it causes visual issues on the flashcard page when it's empty & i want some consistency
        let new_folder_button = widget::button::text(fl!("new"))
            .style(theme::Button::Suggested)
            .on_press(Message::OpenCreateFolderDialog);

        widget::row::with_capacity(2)
            .align_items(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text::title3(fl!("folders")).width(Length::Fill))
            .push(new_folder_button)
            .into()
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.current_studyset_id.is_some() {
            if !self.folders.is_empty() {
                let mut folders = widget::list::list_column()
                    .style(theme::Container::ContextDrawer)
                    .spacing(spacing.space_xxxs)
                    .padding([spacing.space_none, spacing.space_xxs]);

                for folder in &self.folders {
                    // TODO: widget::button::icon
                    let edit_button =
                        widget::button::custom(IconCache::get("edit-button-symbolic", 18))
                            .style(theme::Button::Standard)
                            .on_press(Message::ToggleEditContextPage(Some(folder.clone())));

                    // TODO: widget::button::icon
                    let open_button =
                        widget::button::custom(IconCache::get("folder-open-symbolic", 18))
                            .style(theme::Button::Suggested)
                            .width(Length::Shrink)
                            .on_press(Message::OpenFolder(folder.id.unwrap()));

                    // TODO: widget::button::icon
                    let delete_button =
                        widget::button::custom(IconCache::get("user-trash-full-symbolic", 18))
                            .style(theme::Button::Destructive)
                            .on_press(Message::Delete(folder.id));

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
                widget::column::with_capacity(2)
                    .spacing(spacing.space_xxs)
                    .push(self.folder_header_row())
                    .push(
                        widget::Container::new(
                            widget::Text::new(fl!("empty-page")).size(spacing.space_xl),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(cosmic::iced::alignment::Horizontal::Center)
                        .align_y(cosmic::iced::alignment::Vertical::Center),
                    )
                    .height(Length::Fill)
                    .into()
            }
        } else {
            let column = widget::Column::new()
                .push(
                    widget::Text::new(fl!("empty-page"))
                        .size(spacing.space_xl)
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::Fill),
                )
                .push(
                    widget::Text::new(fl!("empty-page-noset"))
                        .size(spacing.space_l)
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::Fill),
                )
                .width(Length::Fill);

            widget::Container::new(column)
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

        widget::settings::view_column(vec![widget::settings::section()
            .title(fl!("folder-details"))
            .add(
                widget::column::with_children(vec![
                    widget::text::body(fl!("folder-name")).into(),
                    widget::text_input(fl!("folder-name"), &self.new_folder.name)
                        .on_input(Message::NewFolderNameInput)
                        .into(),
                ])
                .spacing(spacing.space_xxs)
                .padding([0, 15, 0, 15]),
            )
            .add(if !self.new_folder.name.is_empty() {
                widget::button::text(fl!("edit"))
                    .on_press(Message::Upsert)
                    .style(theme::Button::Suggested)
                    .width(Length::Fill)
            } else {
                widget::button::text(fl!("edit"))
                    .style(theme::Button::Suggested)
                    .width(Length::Fill)
            })
            .into()])
        .into()
    }
}
