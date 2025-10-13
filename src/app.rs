// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{AppTheme, OboeteConfig};
use crate::key_binds::key_binds;
use crate::oboete::database::init_database;
use crate::oboete::models::flashcard::Flashcard;
use crate::oboete::models::folder::Folder;
use crate::oboete::models::studyset::StudySet;
use crate::oboete::pages::folder_content::{self, FolderContent};
use crate::oboete::pages::homepage::{self, HomePage};
use crate::oboete::pages::study_page::{self, StudyPage};
use crate::oboete::utils::{export_flashcards_json, import_flashcards_json};
use crate::{fl, icons};
use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use cosmic::app::context_drawer;
use cosmic::iced::{Event, Length, Subscription};
use cosmic::iced_core::keyboard::{Key, Modifiers};
use cosmic::prelude::*;
use cosmic::theme;
use cosmic::widget::about::About;
use cosmic::widget::menu::Action;
use cosmic::widget::segmented_button::{self, EntityMut, SingleSelect};
use cosmic::widget::{self, menu, nav_bar};
use sqlx::{Pool, Sqlite};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct Oboete {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Application about page
    about: About,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Dialog Pages of the Application
    dialog_pages: VecDeque<DialogPage>,
    /// Holds the state of the application dialogs
    dialog_state: DialogState,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Currently selected Page
    current_page: Page,
    /// Database of the application
    database: Option<Arc<Pool<Sqlite>>>,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Application Keyboard Modifiers (current state)
    modifiers: Modifiers,
    /// Application configuration handler
    config_handler: Option<cosmic::cosmic_config::Config>,
    // Configuration data that persists between application runs.
    config: OboeteConfig,
    // Application Themes
    app_themes: Vec<String>,
    /// Application HomePage
    homepage: HomePage,
    /// Application FolderContent Page
    folder_content: FolderContent,
    /// Application StudyPage
    study_page: StudyPage,
    /// Contains the data to backup in case a backup is requested
    backup_data: Option<Vec<StudySet>>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    ToggleContextPage(ContextPage),
    UpdateConfig(OboeteConfig),
    UpdateTheme(usize),
    LaunchUrl(String),
    Key(Modifiers, Key),
    Modifiers(Modifiers),

    DatabaseConnected(Arc<Pool<Sqlite>>),

    FetchStudySets,
    PopulateStudySets(Vec<StudySet>),

    UpdatedStudySet(String),
    DeletedStudySet,

    HomePage(homepage::Message),
    FolderContent(folder_content::Message),
    StudyPage(study_page::Message),

    OpenNewStudySetDialog,
    OpenRenameStudySetDialog,
    OpenDeleteStudySetDialog,
    DialogComplete,
    DialogCancel,
    DialogUpdate(DialogPage),

    GetBackupData,
    SetBackupData(Vec<StudySet>),
    OpenSaveBackupFileDialog,
    SaveBackupFile(Vec<String>),

    Import,
    ImportFileResult(Vec<String>),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for Oboete {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = crate::flags::Flags;

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "dev.mariinkys.Oboete";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(core: cosmic::Core, flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Application about page
        let about = About::default()
            .name("Oboete")
            .icon(widget::icon::from_name(Self::APP_ID))
            .version(env!("CARGO_PKG_VERSION"))
            .author("mariinkys")
            .license("GPL-3.0-only")
            .links([
                (fl!("repository"), REPOSITORY),
                (fl!("support"), "https://github.com/mariinkys/oboete/issues"),
            ])
            .developers([("mariinkys", "kysdev.owjga@aleeas.com")])
            .comments("\"Pop Icons\" by System76 is licensed under CC-SA-4.0");

        // Construct the app model with the runtime's core.
        let mut app = Oboete {
            core,
            about,
            context_page: ContextPage::default(),
            dialog_pages: VecDeque::new(),
            dialog_state: DialogState::default(),
            nav: nav_bar::Model::default(),
            current_page: Page::HomePage,
            database: None,
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            homepage: HomePage::init(),
            folder_content: FolderContent::init(),
            study_page: StudyPage::init(),
            backup_data: None,
        };

        let tasks = vec![
            app.update_title(),
            cosmic::command::set_theme(app.config.app_theme.theme()),
            Task::perform(init_database(Self::APP_ID), |database| {
                cosmic::action::app(Message::DatabaseConnected(database))
            }),
        ];

        (app, Task::batch(tasks))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::MenuBar::new(vec![
            menu::Tree::with_children(
                Element::from(menu::root(fl!("file"))),
                menu::items(
                    &self.key_binds,
                    vec![
                        menu::Item::Button(fl!("new-studyset"), None, MenuAction::NewStudySet),
                        menu::Item::Button(fl!("backup"), None, MenuAction::Backup),
                        menu::Item::Button(fl!("import"), None, MenuAction::Import),
                    ],
                ),
            ),
            menu::Tree::with_children(
                Element::from(menu::root(fl!("edit"))),
                menu::items(
                    &self.key_binds,
                    vec![
                        menu::Item::Button(
                            fl!("rename-studyset"),
                            None,
                            MenuAction::RenameStudySet,
                        ),
                        menu::Item::Button(
                            fl!("delete-studyset"),
                            None,
                            MenuAction::DeleteStudySet,
                        ),
                    ],
                ),
            ),
            menu::Tree::with_children(
                Element::from(menu::root(fl!("view"))),
                menu::items(
                    &self.key_binds,
                    vec![
                        menu::Item::Button(fl!("about"), None, MenuAction::About),
                        menu::Item::Button(fl!("settings"), None, MenuAction::Settings),
                    ],
                ),
            ),
        ])
        .item_height(menu::ItemHeight::Dynamic(40))
        .item_width(menu::ItemWidth::Uniform(270))
        .spacing(4.0);

        vec![menu_bar.into()]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |s| Message::LaunchUrl(s.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
            ContextPage::Settings => context_drawer::context_drawer(
                self.settings(),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title(fl!("settings")),
            ContextPage::EditFolder => context_drawer::context_drawer(
                self.homepage
                    .edit_folder_contextpage()
                    .map(Message::HomePage),
                Message::ToggleContextPage(ContextPage::EditFolder),
            )
            .title(fl!("folder-details")),
            ContextPage::AddEditFlashcard => context_drawer::context_drawer(
                self.folder_content
                    .add_edit_flashcard_contextpage()
                    .map(Message::FolderContent),
                Message::ToggleContextPage(ContextPage::AddEditFlashcard),
            )
            .title(fl!("flashcard-options")),
            ContextPage::FolderContentOptions => context_drawer::context_drawer(
                self.folder_content
                    .folder_options_contextpage()
                    .map(Message::FolderContent),
                Message::ToggleContextPage(ContextPage::FolderContentOptions),
            )
            .title(fl!("flashcard-options")),
        })
    }

    /// Display a dialog if requested.
    fn dialog(&self) -> Option<Element<Message>> {
        let dialog_page = self.dialog_pages.front()?;

        let spacing = theme::active().cosmic().spacing;

        // Construct each dialog view
        let dialog = match dialog_page {
            // View of the New StudySet Dialog
            DialogPage::NewStudySet(studyset_name) => widget::dialog()
                .title(fl!("create-studyset"))
                .primary_action(
                    widget::button::suggested(fl!("save")).on_press(Message::DialogComplete),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("studyset-name")).into(),
                        widget::text_input("", studyset_name.as_str())
                            .id(self.dialog_state.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogUpdate(DialogPage::NewStudySet(name))
                            })
                            .on_submit(|_x| Message::DialogComplete)
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),

            // View of the Rename StudySet Dialog
            DialogPage::RenameStudySet { to: studyset_name } => widget::dialog()
                .title(fl!("rename-studyset"))
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
                        widget::text_input("", studyset_name.as_str())
                            .id(self.dialog_state.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogUpdate(DialogPage::RenameStudySet { to: name })
                            })
                            .on_submit(|_x| Message::DialogComplete)
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),

            // View of the DeleteStudySet Dialog
            DialogPage::DeleteStudySet => widget::dialog()
                .title(fl!("delete-studyset"))
                .body(fl!("confirm-delete"))
                .primary_action(
                    widget::button::suggested(fl!("ok"))
                        .on_press_maybe(Some(Message::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                ),

            // View of the NewFolder Dialog
            DialogPage::NewFolder(folder_name) => widget::dialog()
                .title(fl!("create-folder"))
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
                        widget::text_input("", folder_name.as_str())
                            .id(self.dialog_state.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogUpdate(DialogPage::NewFolder(name))
                            })
                            .on_submit(|_x| Message::DialogComplete)
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
        };

        Some(dialog.into())
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<Self::Message> {
        let content = match self.current_page {
            Page::HomePage => self.homepage.view().map(Message::HomePage),
            Page::FolderContent => self.folder_content.view().map(Message::FolderContent),
            Page::StudyPage => self.study_page.view().map(Message::StudyPage),
        };

        widget::Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They are started at the
    /// beginning of the application, and persist through its lifetime.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            // Waych for key_bind inputs
            cosmic::iced::event::listen_with(|event, status, _| match event {
                Event::Keyboard(cosmic::iced::keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                }) => match status {
                    cosmic::iced::event::Status::Ignored => Some(Message::Key(modifiers, key)),
                    cosmic::iced::event::Status::Captured => None,
                },
                Event::Keyboard(cosmic::iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                _ => None,
            }),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<OboeteConfig>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        let mut tasks = vec![];

        match message {
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::UpdateTheme(index) => {
                let app_theme = match index {
                    1 => AppTheme::Dark,
                    2 => AppTheme::Light,
                    _ => AppTheme::System,
                };

                if let Some(handler) = &self.config_handler {
                    if let Err(err) = self.config.set_app_theme(handler, app_theme) {
                        eprintln!("{err}");
                        // even if it fails we update the config (it won't get saved after restart)
                        let mut old_config = self.config.clone();
                        old_config.app_theme = app_theme;
                        self.config = old_config;
                    }

                    return cosmic::command::set_theme(self.config.app_theme.theme());
                }
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },

            Message::Key(modifiers, key) => {
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message());
                    }
                }
            }

            // Updates the current state of keyboard modifiers
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }

            // Sets the database in the appstate and asks to fetch the studysets
            Message::DatabaseConnected(pool) => {
                self.database = Some(pool);
                tasks.push(self.update(Message::FetchStudySets));
            }

            // Fetches the studysets
            Message::FetchStudySets => {
                tasks.push(Task::perform(
                    StudySet::get_all(self.database.clone().unwrap()),
                    |result| match result {
                        Ok(sets) => cosmic::action::app(Message::PopulateStudySets(sets)),
                        Err(_) => cosmic::action::none(),
                    },
                ));
            }

            // Populates the navbar with the fetched StudySets
            Message::PopulateStudySets(sets) => {
                // Reset the navbar items
                self.nav = nav_bar::Model::default();

                // Create a navbar item for each set
                for set in sets {
                    self.create_nav_item(set);
                }
                let Some(entity) = self.nav.iter().next() else {
                    return Task::none();
                };
                self.nav.activate(entity);
                // When a set is clicked on the navbar
                let command = self.on_nav_select(entity);
                tasks.push(command);
            }

            Message::UpdatedStudySet(new_set_name) => {
                let entity = self.nav.active();
                self.nav.text_set(entity, new_set_name);
            }

            // Callback after a StudySet has been successfully deleted from the database
            Message::DeletedStudySet => {
                self.nav.remove(self.nav.active());
                self.homepage.set_current_studyset_id(None);
            }

            // HomePage Commands
            Message::HomePage(message) => {
                let homepage_tasks = self.homepage.update(message);
                for homepage_task in homepage_tasks {
                    match homepage_task {
                        // Fetches the Folders of a given studyset and asks for it to be saved on the homepage state
                        homepage::HomePageTask::FetchSetFolders(set_id) => {
                            tasks.push(Task::perform(
                                Folder::get_all(self.database.clone().unwrap(), set_id),
                                |result| match result {
                                    Ok(folders) => cosmic::action::app(Message::HomePage(
                                        homepage::Message::SetFolders(folders),
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                        }

                        // Edits the given folder with the given data and notifies the HomePage
                        homepage::HomePageTask::EditFolder(folder) => {
                            tasks.push(Task::perform(
                                Folder::edit(self.database.clone().unwrap(), folder),
                                |result| match result {
                                    Ok(_) => cosmic::action::app(Message::HomePage(
                                        homepage::Message::EditedFolder,
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                        }

                        // Deletes the folder with the given id
                        homepage::HomePageTask::DeleteFolder(folder_id) => {
                            tasks.push(Task::perform(
                                Folder::delete(self.database.clone().unwrap(), folder_id),
                                |result| match result {
                                    Ok(_) => cosmic::action::app(Message::HomePage(
                                        homepage::Message::DeletedFolder,
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                        }

                        // Opens the Edit Folder ContextPage
                        homepage::HomePageTask::OpenEditContextPage => {
                            self.context_page = ContextPage::EditFolder;
                            self.core.window.show_context = true;
                        }

                        // Closes any Context Page
                        homepage::HomePageTask::CloseContextPage => {
                            self.context_page = ContextPage::About;
                            self.core.window.show_context = false;
                        }

                        // Opens the Create Folder Dialog
                        homepage::HomePageTask::OpenCreateFolderDialog => {
                            self.dialog_pages
                                .push_back(DialogPage::NewFolder(String::new()));
                            return widget::text_input::focus(
                                self.dialog_state.dialog_text_input.clone(),
                            );
                        }

                        //Opens a folder => Loads the flashcards of a given folder => Updates the current_folder_id
                        homepage::HomePageTask::OpenFolder(folder_id) => {
                            tasks.push(Task::perform(
                                Flashcard::get_all(self.database.clone().unwrap(), folder_id),
                                |result| match result {
                                    Ok(flashcards) => cosmic::action::app(Message::FolderContent(
                                        folder_content::Message::SetFlashcards(flashcards),
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                            self.current_page = Page::FolderContent;
                            self.folder_content.set_current_folder_id(Some(folder_id));
                        }
                    }
                }
            }

            // FolderContent Commands
            Message::FolderContent(message) => {
                let folder_content_tasks = self.folder_content.update(message);
                for folder_content_task in folder_content_tasks {
                    match folder_content_task {
                        // Fetches the Flashcards of a given folder and asks for it to be saved on the folder_content page state
                        folder_content::FolderContentTask::FetchFolderFlashcards(folder_id) => {
                            tasks.push(Task::perform(
                                Flashcard::get_all(self.database.clone().unwrap(), folder_id),
                                |result| match result {
                                    Ok(flashcards) => cosmic::action::app(Message::FolderContent(
                                        folder_content::Message::SetFlashcards(flashcards),
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                        }

                        // Edits the given flashcard with the given data and notifies the folder_content page
                        folder_content::FolderContentTask::EditFlashcard(flashcard) => {
                            tasks.push(Task::perform(
                                Flashcard::edit(self.database.clone().unwrap(), flashcard),
                                |result| match result {
                                    Ok(_) => cosmic::action::app(Message::FolderContent(
                                        folder_content::Message::EditedFlashcard,
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                        }

                        // Adds the given flashcard with the given data and notifies the folder_content page
                        folder_content::FolderContentTask::AddFlashcard(flashcard) => {
                            tasks.push(Task::perform(
                                Flashcard::add(
                                    self.database.clone().unwrap(),
                                    flashcard,
                                    self.folder_content.get_current_folder_id().unwrap(),
                                ),
                                |result| match result {
                                    Ok(_) => cosmic::action::app(Message::FolderContent(
                                        folder_content::Message::AddedNewFlashcard,
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                        }

                        // Deletes the flashcard with the given id
                        folder_content::FolderContentTask::DeleteFlashcard(flashcard_id) => {
                            tasks.push(Task::perform(
                                Flashcard::delete(self.database.clone().unwrap(), flashcard_id),
                                |result| match result {
                                    Ok(_) => cosmic::action::app(Message::FolderContent(
                                        folder_content::Message::DeletedFlashcard,
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                        }

                        // Opens the Add/Edit Flashcard ContextPage
                        folder_content::FolderContentTask::OpenAddEditContextPage => {
                            self.context_page = ContextPage::AddEditFlashcard;
                            self.core.window.show_context = true;
                        }

                        // Closes any Context Page
                        folder_content::FolderContentTask::CloseContextPage => {
                            self.context_page = ContextPage::About;
                            self.core.window.show_context = false;
                        }

                        // Resets the status of a single flashcard and notifies the FolderContent Page
                        folder_content::FolderContentTask::RestartSingleFlashcardStatus(
                            flashcard_id,
                        ) => {
                            tasks.push(Task::perform(
                                Flashcard::reset_single_status(
                                    self.database.clone().unwrap(),
                                    flashcard_id,
                                ),
                                |result| match result {
                                    Ok(_) => cosmic::action::app(Message::FolderContent(
                                        folder_content::Message::EditedFlashcard,
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                        }

                        // Opens the FolderContentOptions ContextPage
                        folder_content::FolderContentTask::OpenFolderOptionsContextPage => {
                            self.context_page = ContextPage::FolderContentOptions;
                            self.core.window.show_context = true;
                        }

                        // Gets a vec of flashcards to get put into the database, does that and notifies the folder_content page
                        folder_content::FolderContentTask::ImportContent(content) => {
                            tasks.push(Task::perform(
                                Flashcard::add_bulk(
                                    self.database.clone().unwrap(),
                                    content,
                                    self.folder_content.get_current_folder_id().unwrap(),
                                ),
                                |result| match result {
                                    Ok(_) => cosmic::action::app(Message::FolderContent(
                                        folder_content::Message::ContentImported,
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                        }

                        // Opens the file selection dialog for anki importing and executes a callback with the result
                        folder_content::FolderContentTask::OpenAnkiFileSelection => {
                            tasks.push(Task::perform(
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
                                    cosmic::action::app(Message::FolderContent(
                                        folder_content::Message::OpenAnkiFileResult(files),
                                    ))
                                },
                            ));
                        }

                        // Resets the status of all flashcards of a given folder and asks to refresh the flashcard vec
                        folder_content::FolderContentTask::RestartFolderFlashcardsStatus(
                            folder_id,
                        ) => {
                            tasks.push(Task::perform(
                                Flashcard::reset_all_status(
                                    self.database.clone().unwrap(),
                                    folder_id,
                                ),
                                |result| match result {
                                    Ok(_) => cosmic::action::app(Message::FolderContent(
                                        folder_content::Message::EditedFlashcard,
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                        }

                        // Opens the export selection dialog for folder exporting and executes a callback with the result
                        folder_content::FolderContentTask::OpenFolderExport(options) => {
                            tasks.push(Task::perform(
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
                                move |files| {
                                    cosmic::action::app(Message::FolderContent(
                                        folder_content::Message::OpenFolderExportResult(
                                            files, options,
                                        ),
                                    ))
                                },
                            ));
                        }

                        // Retrieves the flashcard of a folder and gives it to the studypage
                        folder_content::FolderContentTask::StudyFolder(folder_id) => {
                            tasks.push(Task::perform(
                                Flashcard::get_all(self.database.clone().unwrap(), folder_id),
                                |result| match result {
                                    Ok(flashcards) => cosmic::action::app(Message::StudyPage(
                                        study_page::Message::SetFlashcards(flashcards),
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                            self.current_page = Page::StudyPage;
                        }
                    }
                }
            }

            // StudyPage Commands
            Message::StudyPage(message) => {
                let studypage_tasks = self.study_page.update(message);
                for studypage_task in studypage_tasks {
                    match studypage_task {
                        // Update the flashcard status on the db and return the folder flashcards once again (with the updated status)
                        study_page::StudyPageTask::UpdateFlashcardStatus(flashcard) => {
                            tasks.push(Task::perform(
                                Flashcard::update_status(
                                    self.database.clone().unwrap(),
                                    flashcard,
                                    self.folder_content.get_current_folder_id().unwrap(),
                                ),
                                |result| match result {
                                    Ok(flashcards) => cosmic::action::app(Message::StudyPage(
                                        study_page::Message::UpdatedFlashcardStatus(flashcards),
                                    )),
                                    Err(_) => cosmic::action::none(),
                                },
                            ));
                        }
                    }
                }
            }

            // Opens the dialog page to create a new StudySet
            Message::OpenNewStudySetDialog => {
                self.dialog_pages
                    .push_back(DialogPage::NewStudySet(String::new()));
                return widget::text_input::focus(self.dialog_state.dialog_text_input.clone());
            }

            // Opens the dialog page to rename a studyset
            Message::OpenRenameStudySetDialog => {
                if let Some(set_name) = self.nav.text(self.nav.active()) {
                    self.dialog_pages.push_back(DialogPage::RenameStudySet {
                        to: set_name.to_string(),
                    });
                    return widget::text_input::focus(self.dialog_state.dialog_text_input.clone());
                }
            }

            // Opens the dialog page to delete a studyset
            Message::OpenDeleteStudySetDialog => {
                if self.nav.data::<i32>(self.nav.active()).is_some() {
                    self.dialog_pages.push_back(DialogPage::DeleteStudySet);
                }
            }

            // Executes the action for each dialog page
            Message::DialogComplete => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        // Actions for the NewStudySet Dialog
                        DialogPage::NewStudySet(studyset_name) => {
                            if !studyset_name.is_empty() {
                                let set = StudySet::new(studyset_name);
                                tasks.push(Task::perform(
                                    StudySet::add(self.database.clone().unwrap(), set),
                                    |result| match result {
                                        Ok(_) => cosmic::action::app(Message::FetchStudySets),
                                        Err(_) => cosmic::action::none(),
                                    },
                                ));
                            }
                        }

                        // Actions for the RenameStudySet Dialog
                        DialogPage::RenameStudySet { to: studyset_name } => {
                            if !studyset_name.is_empty() {
                                if let Some(set_id) = self.nav.active_data::<i32>() {
                                    let command = Task::perform(
                                        StudySet::edit(
                                            self.database.clone().unwrap(),
                                            StudySet {
                                                id: Some(*set_id),
                                                name: studyset_name.clone(),
                                                folders: Vec::new(),
                                            },
                                        ),
                                        move |result| match result {
                                            Ok(_) => cosmic::action::app(Message::UpdatedStudySet(
                                                studyset_name.clone(),
                                            )),
                                            Err(_) => cosmic::action::none(),
                                        },
                                    );
                                    tasks.push(command);
                                }
                            }
                        }

                        // Actions for the DeleteStudySet Dialog
                        DialogPage::DeleteStudySet => {
                            if let Some(set_id) = self.nav.data::<i32>(self.nav.active()) {
                                tasks.push(Task::perform(
                                    StudySet::delete(self.database.clone().unwrap(), *set_id),
                                    |result| match result {
                                        Ok(_) => cosmic::action::app(Message::DeletedStudySet),
                                        Err(_) => cosmic::action::none(),
                                    },
                                ));
                            }
                        }

                        // Actions for the NewFolder Dialog
                        DialogPage::NewFolder(folder_name) => {
                            if !folder_name.is_empty() {
                                let folder = Folder::new(folder_name);
                                tasks.push(Task::perform(
                                    Folder::add(
                                        self.database.clone().unwrap(),
                                        folder,
                                        self.homepage.get_current_studyset_id().unwrap(),
                                    ),
                                    |result| match result {
                                        Ok(_folder_id) => cosmic::action::app(Message::HomePage(
                                            homepage::Message::AddedNewFolder,
                                        )),
                                        Err(_) => cosmic::action::none(),
                                    },
                                ));
                            }
                        }
                    }
                }
            }

            // Closes the current dialog
            Message::DialogCancel => {
                self.dialog_pages.pop_front();
            }

            // Updates the current dialog page
            Message::DialogUpdate(dialog_page) => {
                self.dialog_pages[0] = dialog_page;
            }

            // Asks the DB for the data to backup and executes a callback
            Message::GetBackupData => {
                if self.backup_data.is_none() {
                    tasks.push(Task::perform(
                        StudySet::get_all_data(self.database.clone().unwrap()),
                        |result| match result {
                            Ok(data) => cosmic::action::app(Message::SetBackupData(data)),
                            Err(_) => cosmic::action::none(),
                        },
                    ));
                }
            }

            Message::SetBackupData(data) => {
                self.backup_data = Some(data);
                tasks.push(self.update(Message::OpenSaveBackupFileDialog));
            }

            Message::OpenSaveBackupFileDialog => {
                tasks.push(Task::perform(
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
                    |files| cosmic::action::app(Message::SaveBackupFile(files)),
                ));
            }

            Message::SaveBackupFile(open_result) => {
                for path in open_result {
                    if let Some(backup_data) = &self.backup_data {
                        let result = export_flashcards_json(&path, backup_data);
                        match result {
                            Ok(_) => println!("export saved correctly"),
                            Err(err) => eprintln!("{err}"),
                        }
                    }
                }
                self.backup_data = None;
            }

            Message::Import => {
                tasks.push(Task::perform(
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
                    |files| cosmic::action::app(Message::ImportFileResult(files)),
                ));
            }

            Message::ImportFileResult(open_result) => {
                for path in open_result {
                    let result = import_flashcards_json(&path);
                    match result {
                        Ok(studysets) => {
                            let command = Task::perform(
                                StudySet::import(self.database.clone().unwrap(), studysets),
                                |result| match result {
                                    Ok(_) => cosmic::action::app(Message::FetchStudySets),
                                    Err(_) => cosmic::action::app(Message::FetchStudySets),
                                },
                            );

                            tasks.push(command);
                        }
                        Err(err) => eprintln!("{err}"),
                    }
                }
            }
        }

        Task::batch(tasks)
    }

    /// Called when a nav item is selected.
    fn on_nav_select(
        &mut self,
        entity: segmented_button::Entity,
    ) -> Task<cosmic::Action<Self::Message>> {
        let mut tasks = vec![];
        // Activate the page in the model.
        self.nav.activate(entity);

        // Get the data from the navbar (the selected setid)
        let location_opt = self.nav.data::<i32>(entity);
        if let Some(set_id) = location_opt {
            self.current_page = Page::HomePage;
            self.homepage.set_current_studyset_id(Some(*set_id));
            self.folder_content.set_current_folder_id(None);
            self.folder_content.clean_flashcards_vec();

            // If a studyset is clicked ask for the folders of the studyset to be fetched
            let message = Message::HomePage(homepage::Message::FetchSetFolders);
            tasks.push(self.update(message));
        }

        tasks.push(self.update_title());
        Task::batch(tasks)
    }
}

impl Oboete {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = String::from("Oboete");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    pub fn settings(&self) -> Element<Message> {
        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };

        widget::settings::view_column(vec![
            widget::settings::section()
                .title(fl!("appearance"))
                .add(
                    widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                        &self.app_themes,
                        Some(app_theme_selected),
                        Message::UpdateTheme,
                    )),
                )
                .into(),
        ])
        .into()
    }

    fn create_nav_item(&mut self, studyset: StudySet) -> EntityMut<SingleSelect> {
        self.nav
            .insert()
            .text(studyset.name)
            .data(studyset.id.unwrap())
            .icon(icons::get_icon("x-office-document-symbolic", 18))
    }
}

/// The page to display in the application.
#[allow(clippy::enum_variant_names)]
pub enum Page {
    HomePage,
    FolderContent,
    StudyPage,
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
    EditFolder,
    AddEditFlashcard,
    FolderContentOptions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    NewStudySet,
    Backup,
    Import,
    RenameStudySet,
    DeleteStudySet,
    About,
    Settings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::NewStudySet => Message::OpenNewStudySetDialog,
            MenuAction::Backup => Message::GetBackupData,
            MenuAction::Import => Message::Import,
            MenuAction::RenameStudySet => Message::OpenRenameStudySetDialog,
            MenuAction::DeleteStudySet => Message::OpenDeleteStudySetDialog,
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
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

pub struct DialogState {
    /// Input inside of the Dialog Pages of the Application
    dialog_text_input: widget::Id,
}

impl Default for DialogState {
    fn default() -> Self {
        Self {
            dialog_text_input: widget::Id::unique(),
        }
    }
}
