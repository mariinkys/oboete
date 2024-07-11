// SPDX-License-Identifier: GPL-3.0-only

use std::any::TypeId;
use std::collections::{HashMap, VecDeque};

use crate::core::config::{self, AppTheme, CONFIG_VERSION};
use crate::core::database::{
    delete_flashcard, delete_folder, delete_studyset, get_all_data, get_all_studysets,
    get_folder_flashcards, get_single_flashcard, get_single_folder, get_studyset_folders,
    import_flashcards, import_flashcards_to_db, reset_folder_flashcard_status,
    reset_single_flashcard_status, update_flashcard_status, upsert_flashcard, upsert_folder,
    upsert_studyset, OboeteDb,
};
use crate::core::key_bind::key_binds;
use crate::fl;
use crate::flashcards::{self, Flashcards};
use crate::folders::{self, Folders};
use crate::models::{Folder, StudySet};
use crate::utils::{export_flashcards_json, import_flashcards_json, select_random_flashcard};
use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use cosmic::app::{message, Core, Message as CosmicMessage};
use cosmic::iced::{event, keyboard::Event as KeyEvent, Event, Subscription};
use cosmic::iced::{Alignment, Length};
use cosmic::iced_core::keyboard::{Key, Modifiers};
use cosmic::widget::menu::{action::MenuAction, key_bind::KeyBind};
use cosmic::widget::segmented_button::{EntityMut, SingleSelect};
use cosmic::widget::{self, menu, nav_bar, segmented_button};
use cosmic::{cosmic_config, cosmic_theme, theme, Application, ApplicationExt, Command, Element};

const REPOSITORY: &str = "https://github.com/mariinkys/oboete";

pub struct Oboete {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
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
    /// Contains the data to backup in case a backup is requested
    backup_data: Option<Vec<StudySet>>,
    /// Application Themes
    app_themes: Vec<String>,
    /// Config Handler
    config_handler: Option<cosmic_config::Config>,
    /// Application Config
    config: config::OboeteConfig,
    /// Application KeyBinds
    key_binds: HashMap<KeyBind, Action>,
    /// Application Modifiers
    modifiers: Modifiers,
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    DbConnected(OboeteDb),
    AppTheme(usize),
    #[allow(dead_code)]
    SystemThemeModeChange(cosmic_theme::ThemeMode),
    Key(Modifiers, Key),
    Modifiers(Modifiers),

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
    OpenNewFolderDialog,

    Backup,
    SetBackupData(Vec<StudySet>),
    OpenSaveBackupFileDialog,
    SaveBackupFile(Vec<String>),

    Import,
    OpenImportFileResult(Vec<String>),
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
    Settings,
    EditFolder,
    CreateEditFlashcard,
    FlashcardOptions,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::Settings => fl!("settings"),
            Self::EditFolder => fl!("folder-details"),
            Self::CreateEditFlashcard => fl!("flashcard-options"),
            Self::FlashcardOptions => fl!("flashcard-options"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: config::OboeteConfig,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    About,
    Settings,
    NewStudySet,
    RenameStudySet,
    DeleteStudySet,
    Backup,
    Import,
}

impl menu::action::MenuAction for Action {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            Action::About => Message::ToggleContextPage(ContextPage::About),
            Action::Settings => Message::ToggleContextPage(ContextPage::Settings),
            Action::NewStudySet => Message::OpenNewStudySetDialog,
            Action::RenameStudySet => Message::OpenRenameStudySetDialog,
            Action::DeleteStudySet => Message::OpenDeleteStudySetDialog,
            Action::Backup => Message::Backup,
            Action::Import => Message::Import,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    NewStudySet(String),
    RenameStudySet { to: String },
    DeleteStudySet,
    NewFolder(String),
}

impl Application for Oboete {
    type Executor = cosmic::executor::Default;

    type Flags = Flags;

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

    fn init(mut core: Core, flags: Self::Flags) -> (Self, Command<CosmicMessage<Self::Message>>) {
        core.nav_bar_toggle_condensed();
        let nav = segmented_button::ModelBuilder::default().build();

        let app = Oboete {
            core,
            context_page: ContextPage::default(),
            nav,
            current_page: Page::Folders,
            db: None,
            folders: Folders::new(),
            flashcards: Flashcards::new(),
            dialog_pages: VecDeque::new(),
            dialog_text_input: widget::Id::unique(),
            backup_data: None,
            app_themes: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            config_handler: flags.config_handler,
            config: flags.config,
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
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
                menu::root(fl!("file")),
                menu::items(
                    &self.key_binds,
                    vec![
                        menu::Item::Button(fl!("new-studyset"), Action::NewStudySet),
                        menu::Item::Button(fl!("backup"), Action::Backup),
                        menu::Item::Button(fl!("import"), Action::Import),
                    ],
                ),
            ),
            menu::Tree::with_children(
                menu::root(fl!("edit")),
                menu::items(
                    &self.key_binds,
                    vec![
                        menu::Item::Button(fl!("rename-studyset"), Action::RenameStudySet),
                        menu::Item::Button(fl!("delete-studyset"), Action::DeleteStudySet),
                    ],
                ),
            ),
            menu::Tree::with_children(
                menu::root(fl!("view")),
                menu::items(
                    &self.key_binds,
                    vec![
                        menu::Item::Button(fl!("about"), Action::About),
                        menu::Item::Button(fl!("settings"), Action::Settings),
                    ],
                ),
            ),
        ])
        .item_height(menu::ItemHeight::Dynamic(40))
        .item_width(menu::ItemWidth::Uniform(240))
        .spacing(4.0);

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

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;

        let subscriptions = vec![
            event::listen_with(|event, status| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => match status {
                    event::Status::Ignored => Some(Message::Key(modifiers, key)),
                    event::Status::Captured => None,
                },
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                _ => None,
            }),
            cosmic_config::config_subscription(
                TypeId::of::<ConfigSubscription>(),
                Self::APP_ID.into(),
                CONFIG_VERSION,
            )
            .map(|update| {
                if !update.errors.is_empty() {
                    log::info!(
                        "errors loading config {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::SystemThemeModeChange(update.config)
            }),
            cosmic_config::config_subscription::<_, cosmic_theme::ThemeMode>(
                TypeId::of::<ThemeSubscription>(),
                cosmic_theme::THEME_MODE_ID.into(),
                cosmic_theme::ThemeMode::version(),
            )
            .map(|update| {
                if !update.errors.is_empty() {
                    log::info!(
                        "errors loading theme mode {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::SystemThemeModeChange(update.config)
            }),
        ];

        // subscriptions.push(self.content.subscription().map(Message::Content));

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> Command<CosmicMessage<Self::Message>> {
        // Helper for updating config values efficiently
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!(
                                    "failed to save config {:?}: {}",
                                    stringify!($name),
                                    err
                                );
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        log::warn!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name)
                        );
                    }
                }
            };
        }

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
                        folders::Command::OpenCreateFolderDialog => {
                            //TODO: Less terrible way to do this?
                            let command = Command::perform(
                                async { message::app(Message::OpenNewFolderDialog) },
                                |msg| msg,
                            );
                            commands.push(command);
                        }
                        folders::Command::UpsertFolder(folder) => {
                            let command = Command::perform(
                                upsert_folder(
                                    self.db.clone(),
                                    folder,
                                    self.flashcards.current_folder_id,
                                ),
                                |_result| {
                                    message::app(Message::Folders(folders::Message::Upserted))
                                },
                            );
                            self.core.window.show_context = false;
                            commands.push(command);
                        }
                        folders::Command::ToggleEditContextPage(folder) => {
                            if self.context_page == ContextPage::EditFolder {
                                // Close the context drawer if the toggled context page is the same.
                                self.core.window.show_context = !self.core.window.show_context;
                            } else {
                                // Open the context drawer to display the requested context page.
                                self.context_page = ContextPage::EditFolder;
                                self.core.window.show_context = true;
                            }

                            //Loads the flashcard in case is an edit operation
                            if folder.is_some() {
                                let command = Command::perform(
                                    get_single_folder(self.db.clone(), folder.unwrap().id.unwrap()),
                                    |result| match result {
                                        Ok(folder) => message::app(Message::Folders(
                                            folders::Message::LoadedSingle(folder),
                                        )),
                                        Err(_) => message::none(),
                                    },
                                );
                                commands.push(command);
                            }

                            // Set the title of the context drawer.
                            self.set_context_title(ContextPage::EditFolder.title());
                        }
                        folders::Command::DeleteFolder(folder_id) => {
                            let command = Command::perform(
                                delete_folder(self.db.clone(), folder_id.unwrap()),
                                |result| match result {
                                    Ok(_) => message::app(Message::Folders(
                                        folders::Message::LoadFolders,
                                    )),
                                    Err(_) => message::none(),
                                },
                            );
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
                            self.flashcards.currently_studying_flashcard =
                                select_random_flashcard(&self.flashcards.flashcards)
                                    .unwrap_or(crate::models::Flashcard::new_error_variant());
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
                        flashcards::Command::DeleteFlashcard(flashcard_id) => {
                            let command = Command::perform(
                                delete_flashcard(self.db.clone(), flashcard_id.unwrap()),
                                |result| match result {
                                    Ok(_) => message::app(Message::Flashcards(
                                        flashcards::Message::LoadFlashcards,
                                    )),
                                    Err(_) => message::none(),
                                },
                            );
                            commands.push(command);
                        }
                        flashcards::Command::ToggleOptionsPage => {
                            if self.context_page == ContextPage::FlashcardOptions {
                                // Close the context drawer if the toggled context page is the same.
                                self.core.window.show_context = !self.core.window.show_context;
                            } else {
                                // Open the context drawer to display the requested context page.
                                self.context_page = ContextPage::FlashcardOptions;
                                self.core.window.show_context = true;
                            }

                            // Set the title of the context drawer.
                            self.set_context_title(ContextPage::FlashcardOptions.title());
                        }
                        flashcards::Command::ImportFlashcards(flashcards) => {
                            let command = Command::perform(
                                import_flashcards(
                                    self.db.clone(),
                                    flashcards,
                                    self.flashcards.current_folder_id,
                                ),
                                |_result| {
                                    message::app(Message::Flashcards(flashcards::Message::Upserted))
                                },
                            );
                            self.core.window.show_context = false;
                            commands.push(command);
                        }
                        flashcards::Command::RestartSingleFlashcardStatus(flashcard_id) => {
                            let command = Command::perform(
                                reset_single_flashcard_status(self.db.clone(), flashcard_id),
                                |result| match result {
                                    Ok(_) => message::app(Message::Flashcards(
                                        flashcards::Message::LoadFlashcards,
                                    )),
                                    Err(_) => message::none(),
                                },
                            );

                            self.core.window.show_context = false;
                            commands.push(command);
                        }
                        flashcards::Command::RestartFolderFlashcardStatus(folder_id) => {
                            let command = Command::perform(
                                reset_folder_flashcard_status(self.db.clone(), Some(folder_id)),
                                |result| match result {
                                    Ok(_) => message::app(Message::Flashcards(
                                        flashcards::Message::LoadFlashcards,
                                    )),
                                    Err(_) => message::none(),
                                },
                            );

                            self.core.window.show_context = false;
                            commands.push(command);
                        }
                        flashcards::Command::OpenAnkiFileSelection => {
                            let command = Command::perform(
                                async move {
                                    let result = SelectedFiles::open_file()
                                        .title("Open Anki File")
                                        .accept_label("Open")
                                        .modal(true)
                                        .multiple(false)
                                        .filter(FileFilter::new("TXT File").glob("*.txt"))
                                        .send()
                                        .await
                                        .unwrap()
                                        .response();

                                    if let Ok(result) = result {
                                        result
                                            .uris()
                                            .iter()
                                            .map(|file| file.path().to_string())
                                            .collect::<Vec<String>>()
                                    } else {
                                        Vec::new()
                                    }
                                },
                                |files| {
                                    message::app(Message::Flashcards(
                                        flashcards::Message::OpenAnkiFileResult(files),
                                    ))
                                },
                            );
                            commands.push(command);
                        }
                        flashcards::Command::OpenFolderExportDestination(options) => {
                            let command = Command::perform(
                                async move {
                                    let result = SelectedFiles::save_file()
                                        .title("Save Export")
                                        .accept_label("Save")
                                        .modal(true)
                                        .filter(FileFilter::new("TXT File").glob("*.txt"))
                                        .send()
                                        .await
                                        .unwrap()
                                        .response();

                                    if let Ok(result) = result {
                                        result
                                            .uris()
                                            .iter()
                                            .map(|file| file.path().to_string())
                                            .collect::<Vec<String>>()
                                    } else {
                                        Vec::new()
                                    }
                                },
                                |files| {
                                    message::app(Message::Flashcards(
                                        flashcards::Message::OpenFolderExportDestinationResult(
                                            files, options,
                                        ),
                                    ))
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
                            if name.is_empty() == false {
                                let set = StudySet::new(name);
                                commands.push(Command::perform(
                                    upsert_studyset(self.db.clone(), set),
                                    |result| match result {
                                        Ok(set) => message::app(Message::AddStudySet(set)),
                                        Err(_) => message::none(),
                                    },
                                ));
                            }
                        }
                        DialogPage::RenameStudySet { to: name } => {
                            if name.is_empty() == false {
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
                        }
                        DialogPage::DeleteStudySet => {
                            commands.push(self.update(Message::DeleteStudySet));
                        }
                        DialogPage::NewFolder(name) => {
                            if name.is_empty() == false {
                                let folder = Folder::new(name);
                                commands.push(Command::perform(
                                    upsert_folder(
                                        self.db.clone(),
                                        folder,
                                        self.folders.current_studyset_id.unwrap(),
                                    ),
                                    |result| match result {
                                        Ok(_folder_id) => message::app(Message::Folders(
                                            folders::Message::Upserted,
                                        )),
                                        Err(_) => message::none(),
                                    },
                                ));
                            }
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

                    self.folders.current_studyset_id = None;
                    commands.push(self.update(Message::Folders(folders::Message::LoadFolders)));
                    commands.push(command);
                }
                self.nav.remove(self.nav.active());
            }
            Message::OpenNewFolderDialog => {
                self.dialog_pages
                    .push_back(DialogPage::NewFolder(String::new()));
                return widget::text_input::focus(self.dialog_text_input.clone());
            }
            Message::Backup => {
                if self.backup_data.is_none() {
                    let command =
                        Command::perform(get_all_data(self.db.clone()), |result| match result {
                            Ok(data) => message::app(Message::SetBackupData(data)),
                            Err(_) => message::none(),
                        });

                    commands.push(command);
                    commands.push(self.update(Message::OpenSaveBackupFileDialog));
                }
            }
            Message::SetBackupData(data) => {
                self.backup_data = Some(data);
            }
            Message::OpenSaveBackupFileDialog => {
                let command = Command::perform(
                    async move {
                        let result = SelectedFiles::save_file()
                            .title("Save Backup")
                            .accept_label("Save")
                            .modal(true)
                            .filter(FileFilter::new("JSON File").glob("*.json"))
                            .send()
                            .await
                            .unwrap()
                            .response();

                        if let Ok(result) = result {
                            result
                                .uris()
                                .iter()
                                .map(|file| file.path().to_string())
                                .collect::<Vec<String>>()
                        } else {
                            Vec::new()
                        }
                    },
                    |files| message::app(Message::SaveBackupFile(files)),
                );
                commands.push(command);
            }
            Message::SaveBackupFile(open_result) => {
                for path in open_result {
                    if self.backup_data.is_some() {
                        let _result =
                            export_flashcards_json(&path, self.backup_data.clone().unwrap());
                    }
                }
                self.backup_data = None;
            }
            Message::Import => {
                let command = Command::perform(
                    async move {
                        let result = SelectedFiles::open_file()
                            .title("Open Backup File")
                            .accept_label("Open")
                            .modal(true)
                            .multiple(false)
                            .filter(FileFilter::new("JSON File").glob("*.json"))
                            .send()
                            .await
                            .unwrap()
                            .response();

                        if let Ok(result) = result {
                            result
                                .uris()
                                .iter()
                                .map(|file| file.path().to_string())
                                .collect::<Vec<String>>()
                        } else {
                            Vec::new()
                        }
                    },
                    |files| message::app(Message::OpenImportFileResult(files)),
                );
                commands.push(command);
            }
            Message::OpenImportFileResult(open_result) => {
                for path in open_result {
                    let result = import_flashcards_json(&path);
                    match result {
                        Ok(studysets) => {
                            let command = Command::perform(
                                import_flashcards_to_db(self.db.clone(), studysets),
                                |result| match result {
                                    Ok(_) => message::app(Message::FetchStudySets),
                                    Err(_) => message::app(Message::FetchStudySets),
                                },
                            );

                            commands.push(command);
                        }
                        Err(_) => println!("Error importing JSON"),
                    }
                }
            }
            Message::AppTheme(index) => {
                let app_theme = match index {
                    1 => AppTheme::Dark,
                    2 => AppTheme::Light,
                    _ => AppTheme::System,
                };
                config_set!(app_theme, app_theme);
                return self.update_config();
            }
            Message::SystemThemeModeChange(_) => {
                return self.update_config();
            }
            Message::Key(modifiers, key) => {
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message());
                    }
                }
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
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
            ContextPage::Settings => self.settings(),
            ContextPage::EditFolder => self.folders.edit_folder_contextpage().map(Message::Folders),
            ContextPage::CreateEditFlashcard => self
                .flashcards
                .create_edit_flashcard_contextpage()
                .map(Message::Flashcards),
            ContextPage::FlashcardOptions => self
                .flashcards
                .flashcard_options_contextpage()
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
            DialogPage::NewStudySet(name) => widget::dialog(fl!("create-studyset"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press_maybe(Some(Message::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("studyset-name")).into(),
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
            DialogPage::RenameStudySet { to: name } => widget::dialog(fl!("rename-studyset"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press_maybe(Some(Message::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("studyset-name")).into(),
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
            DialogPage::DeleteStudySet => widget::dialog(fl!("delete-studyset"))
                .body(fl!("confirm-delete"))
                .primary_action(
                    widget::button::suggested(fl!("ok"))
                        .on_press_maybe(Some(Message::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                ),
            DialogPage::NewFolder(name) => widget::dialog(fl!("create-folder"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press_maybe(Some(Message::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("folder-name")).into(),
                        widget::text_input("", name.as_str())
                            .id(self.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogUpdate(DialogPage::NewFolder(name))
                            })
                            .on_submit(Message::DialogComplete)
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
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

            let message = Message::Folders(folders::Message::LoadFolders);
            let window_title = format!("Oboete - {}", set.name);

            commands.push(self.set_window_title(window_title.clone()));
            self.set_header_title(window_title);

            return self.update(message);
        }

        Command::batch(commands)
    }
}

impl Oboete {
    fn update_config(&mut self) -> Command<CosmicMessage<Message>> {
        cosmic::app::command::set_theme(self.config.app_theme.theme())
    }

    /// The settings page for this app.
    fn settings(&self) -> Element<Message> {
        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };
        widget::settings::view_column(vec![widget::settings::view_section(fl!("appearance"))
            .add(
                widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                    &self.app_themes,
                    Some(app_theme_selected),
                    Message::AppTheme,
                )),
            )
            .into()])
        .into()
    }

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

        let version_link = widget::button::link(format!("v{}", env!("CARGO_PKG_VERSION")))
            .on_press(Message::LaunchUrl(
                "https://github.com/mariinkys/oboete/releases".to_string(),
            ))
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .push(version_link)
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
