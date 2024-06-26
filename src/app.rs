// SPDX-License-Identifier: GPL-3.0-only

use std::collections::{HashMap, VecDeque};

use crate::core::database::{
    delete_studyset, get_all_studysets, get_folder_flashcards, get_single_flashcard,
    get_studyset_folders, update_flashcard_status, upsert_flashcard, upsert_folder,
    upsert_studyset, OboeteDb,
};
use crate::fl;
use crate::flashcards::{self, Flashcards};
use crate::folders::{self, Folders};
use crate::models::StudySet;
use crate::utils::select_random_flashcard;
use cosmic::app::{message, Core, Message as CosmicMessage};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::segmented_button::{EntityMut, SingleSelect};
use cosmic::widget::{self, menu, nav_bar, segmented_button};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Command, Element};

const REPOSITORY: &str = "https://github.com/mariinkys/oboete";

pub struct Oboete {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// A model that contains all of the pages assigned to the nav bar panel.
    nav: segmented_button::SingleSelectModel,
    /// Currently selected Page
    current_page: Page,
    /// Dialog Pages of the Application
    dialog_pages: VecDeque<DialogPage>,
    /// Input inside of the Dialog Pages of the Application
    dialog_text_input: widget::Id,
    /// Database of the application
    db: Option<OboeteDb>,
    /// Folders Page
    folders: Folders,
    /// Flashcards Page (A folder flashcards, not all flashcards)
    flashcards: Flashcards,
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    DbConnected(OboeteDb),
    Folders(folders::Message),
    Flashcards(flashcards::Message),
    FetchStudySets,
    PopulateStudySets(Vec<StudySet>),
    OpenNewStudySetDialog,
    OpenRenameStudySetDialog,
    OpenDeleteStudySetDialog,
    DialogCancel,
    DialogComplete,
    DialogUpdate(DialogPage),
    AddStudySet(StudySet),
    DeleteStudySet,
}

/// Identifies a page in the application.
pub enum Page {
    Folders,
    FolderFlashcards,
    StudyFolderFlashcards,
}

/// Identifies a context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    NewFolder,
    CreateEditFlashcard,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::NewFolder => fl!("new-folder"),
            Self::CreateEditFlashcard => fl!("flashcard-details"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    NewStudySet,
    RenameStudySet,
    DeleteStudySet,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::NewStudySet => Message::OpenNewStudySetDialog,
            MenuAction::RenameStudySet => Message::OpenRenameStudySetDialog,
            MenuAction::DeleteStudySet => Message::OpenDeleteStudySetDialog,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    NewStudySet(String),
    RenameStudySet { to: String },
    DeleteStudySet,
}

impl Application for Oboete {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "dev.mariinkys.Oboete";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Instructs the cosmic runtime to use this model as the nav bar model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn init(mut core: Core, _flags: Self::Flags) -> (Self, Command<CosmicMessage<Self::Message>>) {
        core.nav_bar_toggle_condensed();
        let nav = segmented_button::ModelBuilder::default().build();

        let app = Oboete {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            nav,
            current_page: Page::Folders,
            db: None,
            folders: Folders::new(),
            flashcards: Flashcards::new(),
            dialog_pages: VecDeque::new(),
            dialog_text_input: widget::Id::unique(),
        };

        //Connect to the Database and Run the needed migrations
        let commands = vec![Command::perform(OboeteDb::init(Self::APP_ID), |database| {
            message::app(Message::DbConnected(database))
        })];

        (app, Command::batch(commands))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![
            menu::Tree::with_children(
                menu::root("File"),
                menu::items(
                    &self.key_binds,
                    vec![menu::Item::Button("New StudySet", MenuAction::NewStudySet)],
                ),
            ),
            menu::Tree::with_children(
                menu::root("Edit"),
                menu::items(
                    &self.key_binds,
                    vec![
                        menu::Item::Button("Rename StudySet", MenuAction::RenameStudySet),
                        menu::Item::Button("Delete StudySet", MenuAction::DeleteStudySet),
                    ],
                ),
            ),
            menu::Tree::with_children(
                menu::root(fl!("view")),
                menu::items(
                    &self.key_binds,
                    vec![menu::Item::Button(fl!("about"), MenuAction::About)],
                ),
            ),
        ]);

        vec![menu_bar.into()]
    }

    fn view(&self) -> Element<Self::Message> {
        let content = match self.current_page {
            Page::Folders => self.folders.view().map(Message::Folders),
            Page::FolderFlashcards => self.flashcards.view().map(Message::Flashcards),
            Page::StudyFolderFlashcards => {
                self.flashcards.view_study_page().map(Message::Flashcards)
            }
        };

        widget::Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Command<CosmicMessage<Self::Message>> {
        let mut commands = vec![];

        match message {
            Message::LaunchUrl(url) => {
                let _result = open::that_detached(url);
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }

                // Set the title of the context drawer.
                self.set_context_title(context_page.title());
            }
            Message::DbConnected(db) => {
                self.db = Some(db);
                let command = self.update(Message::FetchStudySets);
                commands.push(command);
            }
            Message::Folders(message) => {
                let folder_commands = self.folders.update(message);

                for folder_command in folder_commands {
                    match folder_command {
                        //Loads the folders of a given studyset
                        folders::Command::LoadFolders(studyset_id) => {
                            let command = Command::perform(
                                get_studyset_folders(self.db.clone(), studyset_id),
                                |result| match result {
                                    Ok(folders) => message::app(Message::Folders(
                                        folders::Message::SetFolders(folders),
                                    )),
                                    Err(_) => message::none(),
                                },
                            );

                            commands.push(command);
                        }
                        //Opens the NewFolder ContextPage
                        folders::Command::ToggleCreateFolderPage => {
                            if self.context_page == ContextPage::NewFolder {
                                // Close the context drawer if the toggled context page is the same.
                                self.core.window.show_context = !self.core.window.show_context;
                            } else {
                                // Open the context drawer to display the requested context page.
                                self.context_page = ContextPage::NewFolder;
                                self.core.window.show_context = true;
                            }

                            // Set the title of the context drawer.
                            self.set_context_title(ContextPage::NewFolder.title());
                        }
                        //Creates a Folder inside a StudySet
                        folders::Command::CreateFolder(folder) => {
                            let command = Command::perform(
                                upsert_folder(
                                    self.db.clone(),
                                    folder,
                                    //TODO: Safe to unwrap?
                                    self.folders.current_studyset_id.unwrap(),
                                ),
                                |_result| message::app(Message::Folders(folders::Message::Created)),
                            );
                            self.core.window.show_context = false;
                            commands.push(command);
                        }
                        //Opens a folder => Loads the flashcards of a given folder => Updates the current_folder_id
                        folders::Command::OpenFolder(folder_id) => {
                            let command = Command::perform(
                                get_folder_flashcards(self.db.clone(), folder_id),
                                |result| match result {
                                    Ok(flashcards) => message::app(Message::Flashcards(
                                        flashcards::Message::SetFlashcards(flashcards),
                                    )),
                                    Err(_) => message::none(),
                                },
                            );
                            self.current_page = Page::FolderFlashcards;
                            self.flashcards.current_folder_id = folder_id;

                            commands.push(command);
                        }
                    }
                }
            }
            Message::Flashcards(message) => {
                let flashcard_commands = self.flashcards.update(message);

                for flashcard_command in flashcard_commands {
                    match flashcard_command {
                        //Loads the flashcards of a given folder
                        flashcards::Command::LoadFlashcards(folder_id) => {
                            let command = Command::perform(
                                get_folder_flashcards(self.db.clone(), folder_id),
                                |result| match result {
                                    Ok(flashcards) => message::app(Message::Flashcards(
                                        flashcards::Message::SetFlashcards(flashcards),
                                    )),
                                    Err(_) => message::none(),
                                },
                            );

                            commands.push(command);
                        }
                        //Opens the NewFlashcard ContextPage
                        flashcards::Command::ToggleCreateFlashcardPage(flashcard) => {
                            if self.context_page == ContextPage::CreateEditFlashcard {
                                // Close the context drawer if the toggled context page is the same.
                                self.core.window.show_context = !self.core.window.show_context;
                            } else {
                                // Open the context drawer to display the requested context page.
                                self.context_page = ContextPage::CreateEditFlashcard;
                                self.core.window.show_context = true;
                            }

                            //Loads the flashcard in case is an edit operation
                            if flashcard.is_some() {
                                let command = Command::perform(
                                    get_single_flashcard(
                                        self.db.clone(),
                                        flashcard.unwrap().id.unwrap(),
                                    ),
                                    |result| match result {
                                        Ok(flashcard) => message::app(Message::Flashcards(
                                            flashcards::Message::LoadedSingle(flashcard),
                                        )),
                                        Err(_) => message::none(),
                                    },
                                );
                                commands.push(command);
                            }

                            // Set the title of the context drawer.
                            self.set_context_title(ContextPage::CreateEditFlashcard.title());
                        }
                        //Upserts a Flashcard inside a Folder
                        flashcards::Command::UpsertFlashcard(flashcard) => {
                            let command = Command::perform(
                                upsert_flashcard(
                                    self.db.clone(),
                                    flashcard,
                                    self.flashcards.current_folder_id,
                                ),
                                |_result| {
                                    message::app(Message::Flashcards(flashcards::Message::Upserted))
                                },
                            );
                            self.core.window.show_context = false;
                            commands.push(command);
                        }
                        //We select a random (weighted) flashcard and open the page
                        flashcards::Command::OpenStudyFolderFlashcardsPage => {
                            self.flashcards.currently_studying_flashcard = select_random_flashcard(
                                &self.flashcards.flashcards,
                            )
                            .unwrap_or(crate::models::Flashcard {
                                id: None,
                                front: String::from("Error"),
                                back: String::from("Error"),
                                status: 0,
                            });
                            self.current_page = Page::StudyFolderFlashcards
                        }
                        //Update the status on the db and return the folder flashcards once again (with the updated status)
                        flashcards::Command::UpdateFlashcardStatus(flashcard) => {
                            let command = Command::perform(
                                update_flashcard_status(
                                    self.db.clone(),
                                    flashcard,
                                    self.flashcards.current_folder_id,
                                ),
                                |result| match result {
                                    Ok(flashcards) => message::app(Message::Flashcards(
                                        flashcards::Message::UpdatedStatus(flashcards),
                                    )),
                                    Err(_) => message::none(),
                                },
                            );
                            commands.push(command);
                        }
                    }
                }
            }
            Message::FetchStudySets => {
                commands.push(Command::perform(
                    get_all_studysets(self.db.clone()),
                    |result| match result {
                        Ok(data) => message::app(Message::PopulateStudySets(data)),
                        Err(_) => message::none(),
                    },
                ));
            }
            Message::PopulateStudySets(studysets) => {
                for set in studysets {
                    self.create_nav_item(set);
                }
                let Some(entity) = self.nav.iter().next() else {
                    return Command::none();
                };
                self.nav.activate(entity);
                let command = self.on_nav_select(entity);
                commands.push(command);
            }
            Message::OpenNewStudySetDialog => {
                self.dialog_pages
                    .push_back(DialogPage::NewStudySet(String::new()));
                return widget::text_input::focus(self.dialog_text_input.clone());
            }
            Message::OpenRenameStudySetDialog => {
                if let Some(set) = self.nav.data::<StudySet>(self.nav.active()) {
                    self.dialog_pages.push_back(DialogPage::RenameStudySet {
                        to: set.name.clone(),
                    });
                    return widget::text_input::focus(self.dialog_text_input.clone());
                }
            }
            Message::OpenDeleteStudySetDialog => {
                if self.nav.data::<StudySet>(self.nav.active()).is_some() {
                    self.dialog_pages.push_back(DialogPage::DeleteStudySet);
                }
            }
            Message::DialogComplete => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::NewStudySet(name) => {
                            //TODO
                            //let set = StudySet::new(&name);
                            let set = StudySet {
                                id: None,
                                name,
                                folders: Vec::new(),
                            };
                            commands.push(Command::perform(
                                upsert_studyset(self.db.clone(), set),
                                |result| match result {
                                    Ok(set) => message::app(Message::AddStudySet(set)),
                                    Err(_) => message::none(),
                                },
                            ));
                        }
                        DialogPage::RenameStudySet { to: name } => {
                            let entity = self.nav.active();
                            self.nav.text_set(entity, name.clone());
                            if let Some(set) = self.nav.active_data_mut::<StudySet>() {
                                set.name = name.clone();
                                let command = Command::perform(
                                    upsert_studyset(self.db.clone(), set.to_owned().clone()),
                                    |_| message::none(),
                                );
                                commands.push(command);
                            }
                        }
                        DialogPage::DeleteStudySet => {
                            commands.push(self.update(Message::DeleteStudySet));
                        }
                    }
                }
            }
            Message::DialogUpdate(dialog_page) => {
                self.dialog_pages[0] = dialog_page;
            }
            Message::DialogCancel => {
                self.dialog_pages.pop_front();
            }
            Message::AddStudySet(set) => {
                self.create_nav_item(set);
                let Some(entity) = self.nav.iter().last() else {
                    return Command::none();
                };
                let command = self.on_nav_select(entity);
                commands.push(command);
            }
            Message::DeleteStudySet => {
                if let Some(set) = self.nav.data::<StudySet>(self.nav.active()) {
                    let command = Command::perform(
                        delete_studyset(self.db.clone(), set.id.unwrap()),
                        |result| match result {
                            Ok(_) => message::none(),
                            Err(_) => message::none(),
                        },
                    );

                    commands.push(self.update(Message::Folders(folders::Message::Load(None))));
                    commands.push(command);
                }
                self.nav.remove(self.nav.active());
            }
        }

        Command::batch(commands)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
            ContextPage::NewFolder => self.folders.new_folder_contextpage().map(Message::Folders),
            ContextPage::CreateEditFlashcard => self
                .flashcards
                .create_edit_flashcard_contextpage()
                .map(Message::Flashcards),
        })
    }

    fn dialog(&self) -> Option<Element<Message>> {
        let dialog_page = match self.dialog_pages.front() {
            Some(some) => some,
            None => return None,
        };

        let spacing = theme::active().cosmic().spacing;

        let dialog = match dialog_page {
            DialogPage::NewStudySet(name) => widget::dialog("Create StudySet")
                .primary_action(
                    widget::button::suggested("Save").on_press_maybe(Some(Message::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard("Cancel").on_press(Message::DialogCancel),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body("StudySet Name").into(),
                        widget::text_input("", name.as_str())
                            .id(self.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogUpdate(DialogPage::NewStudySet(name))
                            })
                            .on_submit(Message::DialogComplete)
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::RenameStudySet { to: name } => widget::dialog("Rename StudySet")
                .primary_action(
                    widget::button::suggested("Save").on_press_maybe(Some(Message::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard("Cancel").on_press(Message::DialogCancel),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body("StudySet Name").into(),
                        widget::text_input("", name.as_str())
                            .id(self.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogUpdate(DialogPage::RenameStudySet { to: name })
                            })
                            .on_submit(Message::DialogComplete)
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::DeleteStudySet => widget::dialog("Delete StudySet")
                .body("Confirm Delete")
                .primary_action(
                    widget::button::suggested("Ok").on_press_maybe(Some(Message::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard("Cancel").on_press(Message::DialogCancel),
                ),
        };

        Some(dialog.into())
    }

    /// Called when a nav item is selected.
    fn on_nav_select(
        &mut self,
        entity: segmented_button::Entity,
    ) -> Command<CosmicMessage<Self::Message>> {
        let mut commands = vec![];
        self.nav.activate(entity);
        let location_opt = self.nav.data::<StudySet>(entity);

        if let Some(set) = location_opt {
            self.current_page = Page::Folders;
            self.folders.current_studyset_id = set.id;

            let message = Message::Folders(folders::Message::Load(set.id));
            let window_title = format!("Oboete - {}", set.name);

            commands.push(self.set_window_title(window_title.clone()));
            self.set_header_title(window_title);

            return self.update(message);
        }

        Command::batch(commands)
    }
}

impl Oboete {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(
            &include_bytes!("../res/icons/hicolor/128x128/apps/dev.mariinkys.Oboete.svg")[..],
        ));

        let title = widget::text::title3(fl!("app-title"));

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::LaunchUrl(REPOSITORY.to_string()))
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    fn create_nav_item(&mut self, studyset: StudySet) -> EntityMut<SingleSelect> {
        self.nav
            .insert()
            .text(studyset.name.clone())
            .data(studyset.clone())
    }
}
