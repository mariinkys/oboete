// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use crate::core::database::{
    get_all_studysets, get_folder_flashcards, get_single_flashcard, get_studyset_folders,
    update_flashcard_status, upsert_flashcard, upsert_folder, upsert_studyset, OboeteDb,
};
use crate::flashcards::{self, Flashcards};
use crate::folders::{self, Folders};
use crate::studysets::StudySets;
use crate::utils::select_random_flashcard;
use crate::{fl, studysets};
use cosmic::app::{message, Core, Message as CosmicMessage};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, icon, menu, nav_bar};
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
    nav: nav_bar::Model,
    /// Currently selected Page
    current_page: Page,
    /// Database of the application
    db: Option<OboeteDb>,
    /// StudySets Page
    studysets: StudySets,
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
    StudySets(studysets::Message),
    Folders(folders::Message),
    Flashcards(flashcards::Message),
}

/// Identifies a page in the application.
pub enum Page {
    StudySets,
    Folders,
    FolderFlashcards,
    StudyFolderFlashcards,
    AllFlashcards,
}

/// Identifies a context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    NewStudySet,
    NewFolder,
    CreateEditFlashcard,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::NewStudySet => fl!("new-studyset"),
            Self::NewFolder => fl!("new-folder"),
            Self::CreateEditFlashcard => fl!("flashcard-details"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
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

    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<CosmicMessage<Self::Message>>) {
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text("Study Sets")
            .data::<Page>(Page::StudySets)
            .icon(icon::from_name("applications-science-symbolic"))
            .activate();

        nav.insert()
            .text("All Flashcards")
            .data::<Page>(Page::AllFlashcards)
            .icon(icon::from_name("applications-system-symbolic"));

        let mut app = Oboete {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            nav,
            current_page: Page::StudySets,
            db: None,
            studysets: StudySets::new(),
            folders: Folders::new(),
            flashcards: Flashcards::new(),
        };

        //Connect to the Database and Run the needed migrations
        let commands = vec![
            Command::perform(OboeteDb::init(Self::APP_ID), |database| {
                message::app(Message::DbConnected(database))
            }),
            app.update_titles(),
        ];

        (app, Command::batch(commands))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn view(&self) -> Element<Self::Message> {
        let content = match self.current_page {
            Page::StudySets => self.studysets.view().map(Message::StudySets),
            Page::Folders => self.folders.view().map(Message::Folders),
            Page::FolderFlashcards => self.flashcards.view().map(Message::Flashcards),
            Page::StudyFolderFlashcards => {
                self.flashcards.view_study_page().map(Message::Flashcards)
            }
            Page::AllFlashcards => todo!(),
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
                //TODO: How to not clone the DB for every operation
                self.db = Some(db);
                let command = self.update(Message::StudySets(studysets::Message::GetStudySets));
                commands.push(command);
            }
            Message::StudySets(message) => {
                let studyset_commands = self.studysets.update(message);

                for studyset_command in studyset_commands {
                    match studyset_command {
                        //Opens the NewStudySet ContextPage
                        studysets::Command::ToggleCreateStudySetPage => {
                            if self.context_page == ContextPage::NewStudySet {
                                // Close the context drawer if the toggled context page is the same.
                                self.core.window.show_context = !self.core.window.show_context;
                            } else {
                                // Open the context drawer to display the requested context page.
                                self.context_page = ContextPage::NewStudySet;
                                self.core.window.show_context = true;
                            }

                            // Set the title of the context drawer.
                            self.set_context_title(ContextPage::NewStudySet.title());
                        }
                        //Creates a StudySet
                        studysets::Command::CreateStudySet(studyset) => {
                            let command = Command::perform(
                                upsert_studyset(self.db.clone(), studyset),
                                |_result| {
                                    message::app(Message::StudySets(studysets::Message::Created))
                                },
                            );
                            self.core.window.show_context = false;
                            commands.push(command);
                        }
                        //Loads the studysets
                        studysets::Command::LoadStudySets => {
                            let command =
                                Command::perform(get_all_studysets(self.db.clone()), |result| {
                                    match result {
                                        Ok(studysets) => message::app(Message::StudySets(
                                            studysets::Message::SetStudySets(studysets),
                                        )),
                                        Err(_) => message::none(),
                                    }
                                });

                            commands.push(command);
                        }
                        //Opens a studyset => Loads the folders of a given studyset => Updates the current_studyset_id
                        studysets::Command::OpenStudySet(studyset_id) => {
                            let command = Command::perform(
                                get_studyset_folders(self.db.clone(), studyset_id),
                                |result| match result {
                                    Ok(folders) => message::app(Message::Folders(
                                        folders::Message::SetFolders(folders),
                                    )),
                                    Err(_) => message::none(),
                                },
                            );
                            self.current_page = Page::Folders;
                            self.folders.current_studyset_id = studyset_id;

                            commands.push(command);
                        }
                    }
                }
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
                                    self.folders.current_studyset_id,
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
            ContextPage::NewStudySet => self
                .studysets
                .new_studyset_contextpage()
                .map(Message::StudySets),
            ContextPage::NewFolder => self.folders.new_folder_contextpage().map(Message::Folders),
            ContextPage::CreateEditFlashcard => self
                .flashcards
                .create_edit_flashcard_contextpage()
                .map(Message::Flashcards),
        })
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Command<CosmicMessage<Self::Message>> {
        // Activate the page in the model.
        self.nav.activate(id);

        //Update the current page
        let current_page: Option<&Page> = self.nav.active_data();
        match current_page {
            Some(page) => match page {
                Page::StudySets => self.current_page = Page::StudySets,
                Page::Folders => self.current_page = Page::Folders,
                Page::FolderFlashcards => self.current_page = Page::FolderFlashcards,
                Page::AllFlashcards => self.current_page = Page::AllFlashcards,
                Page::StudyFolderFlashcards => self.current_page = Page::StudyFolderFlashcards,
            },
            None => self.current_page = Page::StudySets,
        }

        self.update_titles()
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

    /// Updates the header and window titles.
    pub fn update_titles(&mut self) -> Command<CosmicMessage<Message>> {
        let mut window_title = fl!("app-title");
        let mut header_title = String::new();

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
            header_title.push_str(page);
        }

        self.set_header_title(header_title);
        self.set_window_title(window_title)
    }
}
