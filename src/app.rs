// SPDX-License-Identifier: GPL-3.0

use crate::app::app_menu::MenuAction;
use crate::app::context_page::ContextPage;
use crate::app::core::init_database;
use crate::app::core::models::studyset::StudySet;
use crate::app::dialogs::{DialogPage, DialogState};
use crate::app::screen::{Screen, flashcards, folders, study};
use crate::config::{AppTheme, OboeteConfig};
use crate::key_binds::key_binds;
use crate::{fl, icons};
use cosmic::app::context_drawer;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Event, Length, Subscription};
use cosmic::iced_core::keyboard::{Key, Modifiers};
use cosmic::iced_widget::center;
use cosmic::prelude::*;
use cosmic::widget::menu::Action;
use cosmic::widget::{self, about::About, menu, nav_bar};
use cosmic::widget::{segmented_button, text};
use sqlx::{Pool, Sqlite};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

pub mod app_menu;
mod context_page;
mod core;
mod dialogs;
mod screen;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Dialog Pages of the Application
    dialog_pages: VecDeque<DialogPage>,
    /// Holds the state of the application dialogs
    dialog_state: DialogState,
    /// The about page for this app.
    about: About,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Application Keyboard Modifiers
    modifiers: Modifiers,
    /// Application configuration handler
    config_handler: Option<cosmic::cosmic_config::Config>,
    /// Configuration data that persists between application runs.
    config: OboeteConfig,
    // Application Themes
    app_themes: Vec<String>,
    /// Application State
    state: State,
}

#[allow(clippy::large_enum_variant)]
enum State {
    Loading,
    Ready {
        /// Application Database
        database: Arc<Pool<Sqlite>>,
        /// Application Screen
        screen: Screen,
    },
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(OboeteConfig),
    UpdateTheme(usize),
    MenuAction(app_menu::MenuAction),
    DialogAction(dialogs::DialogAction),
    Key(Modifiers, Key),
    Modifiers(Modifiers),

    DatabaseLoaded(Arc<Pool<Sqlite>>),

    FetchStudySets,
    FetchedStudySets(Result<Vec<StudySet>, anywho::Error>),

    Folders(folders::Message),
    Flashcards(flashcards::Message),
    Study(study::Message),

    OpenFolders(i32),
    OpenFlashcards(i32),
    OpenStudy(i32),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
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
        // Create the about widget
        let about = About::default()
            .name("Oboete")
            .icon(widget::icon::from_name(Self::APP_ID))
            .version(env!("CARGO_PKG_VERSION"))
            .links([
                (fl!("repository"), REPOSITORY),
                (fl!("support"), &format!("{}/issues", REPOSITORY)),
            ])
            .license(env!("CARGO_PKG_LICENSE"))
            .author("mariinkys")
            .developers([("mariinkys", "kysdev.owjga@aleeas.com")])
            .comments("\"Pop Icons\" by System76 is licensed under CC-SA-4.0");

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            dialog_pages: VecDeque::default(),
            dialog_state: DialogState::default(),
            about,
            nav: nav_bar::Model::default(),
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            state: State::Loading,
        };

        // Startup tasks.
        let tasks = vec![
            app.update_title(),
            cosmic::command::set_theme(app.config.app_theme.theme()),
            Task::perform(init_database(Self::APP_ID), |database| {
                cosmic::action::app(Message::DatabaseLoaded(database))
            }),
        ];

        (app, Task::batch(tasks))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
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
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        self.context_page.display(self)
    }

    /// Display a dialog if requested.
    fn dialog(&self) -> Option<Element<Message>> {
        let dialog_page = self.dialog_pages.front()?;
        dialog_page.display(&self.dialog_state)
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<_> = match &self.state {
            State::Loading => center(text("Loading...")).into(),
            State::Ready { screen, .. } => match screen {
                Screen::Folders(folders_screen) => folders_screen.view().map(Message::Folders),
                Screen::Flashcards(flashcards_screen) => {
                    flashcards_screen.view().map(Message::Flashcards)
                }
                Screen::Study(study_screen) => study_screen.view().map(Message::Study),
            },
        };

        widget::container(content)
            .height(Length::Fill)
            .apply(widget::container)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let mut subscriptions = vec![
            // Watch for key_bind inputs
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
        ];

        let State::Ready { screen, .. } = &self.state else {
            return Subscription::batch(subscriptions);
        };

        match screen {
            Screen::Folders(folders_screen) => {
                subscriptions.push(folders_screen.subscription().map(Message::Folders))
            }
            Screen::Flashcards(flashcards_screen) => {
                subscriptions.push(flashcards_screen.subscription().map(Message::Flashcards))
            }
            Screen::Study(study_screen) => {
                subscriptions.push(study_screen.subscription().map(Message::Study))
            }
        };

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
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
                Task::none()
            }
            Message::UpdateConfig(config) => {
                self.config = config;
                Task::none()
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
                Task::none()
            }
            Message::LaunchUrl(url) => {
                match open::that_detached(&url) {
                    Ok(()) => {}
                    Err(err) => {
                        eprintln!("failed to open {url:?}: {err}");
                    }
                }
                Task::none()
            }
            Message::MenuAction(action) => {
                let State::Ready { .. } = &mut self.state else {
                    return Task::none();
                };

                match action {
                    MenuAction::About => {
                        self.update(Message::ToggleContextPage(ContextPage::About))
                    }
                    MenuAction::NewStudySet => self.update(Message::DialogAction(
                        dialogs::DialogAction::OpenNewStudySetDialog,
                    )),
                    MenuAction::Backup => todo!(),
                    MenuAction::Import => todo!(),
                    MenuAction::RenameStudySet => self.update(Message::DialogAction(
                        dialogs::DialogAction::OpenRenameStudySetDialog,
                    )),
                    MenuAction::DeleteStudySet => self.update(Message::DialogAction(
                        dialogs::DialogAction::OpenDeleteStudySetDialog,
                    )),
                    MenuAction::Settings => {
                        self.update(Message::ToggleContextPage(ContextPage::Settings))
                    }
                }
            }
            Message::DialogAction(action) => {
                let State::Ready { database, .. } = &mut self.state else {
                    return Task::none();
                };

                action.execute(
                    &mut self.dialog_pages,
                    &self.dialog_state,
                    database,
                    &self.nav,
                )
            }
            Message::Key(modifiers, key) => {
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message());
                    }
                }
                Task::none()
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
                Task::none()
            }

            Message::DatabaseLoaded(pool) => {
                let (folders, _task) = screen::FoldersScreen::new(&Arc::clone(&pool), None);

                self.state = State::Ready {
                    database: pool,
                    screen: Screen::Folders(folders),
                };

                //let load_studysets = self.update(Message::FetchStudySets);
                //let folders_tasks = task.map(|msg| cosmic::action::app(Message::Folders(msg)));

                //load_studysets.chain(folders_tasks)
                self.update(Message::FetchStudySets)
            }

            Message::FetchStudySets => {
                let State::Ready { database, .. } = &mut self.state else {
                    return Task::none();
                };

                Task::perform(StudySet::get_all(Arc::clone(database)), |r| {
                    cosmic::action::app(Message::FetchedStudySets(r))
                })
            }
            Message::FetchedStudySets(result) => {
                match result {
                    Ok(sets) => {
                        // Reset the navbar items
                        self.nav = nav_bar::Model::default();

                        for set in sets {
                            self.nav
                                .insert()
                                .text(set.name)
                                .data(set.id.unwrap())
                                .icon(icons::get_icon("x-office-document-symbolic", 18));
                        }

                        // If there's no study sets on the navbar.
                        let Some(entity) = self.nav.iter().next() else {
                            let State::Ready {
                                screen, database, ..
                            } = &mut self.state
                            else {
                                return Task::none();
                            };

                            let (folders, _task) =
                                screen::FoldersScreen::new(&Arc::clone(database), None);
                            *screen = Screen::Folders(folders);

                            return Task::none();
                        };

                        // If there are any items on the navbar...
                        self.nav.activate(entity);
                        self.on_nav_select(entity)
                    }
                    Err(e) => {
                        //TODO: Handle error
                        eprintln!("{}", e);
                        Task::none()
                    }
                }
            }

            Message::Folders(message) => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let Screen::Folders(folders) = screen else {
                    return Task::none();
                };

                match folders.update(message, database) {
                    folders::Action::None => Task::none(),
                    folders::Action::Run(task) => {
                        task.map(|msg| cosmic::action::app(Message::Folders(msg)))
                    }

                    folders::Action::OpenCreateFolderDialog => self.update(Message::DialogAction(
                        dialogs::DialogAction::OpenCreateFolderDialog,
                    )),
                    folders::Action::OpenDeleteFolderDialog(folder_id) => {
                        self.update(Message::DialogAction(
                            dialogs::DialogAction::OpenDeleteFolderDialog(folder_id),
                        ))
                    }
                    folders::Action::OpenContextPage(context_page) => {
                        self.update(Message::ToggleContextPage(context_page))
                    }
                    folders::Action::OpenFolder(folder_id) => {
                        self.update(Message::OpenFlashcards(folder_id))
                    }
                }
            }
            Message::OpenFolders(studyset_id) => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let (folders, task) = screen::FoldersScreen::new(database, Some(studyset_id));
                *screen = Screen::Folders(folders);
                task.map(|msg| cosmic::action::app(Message::Folders(msg)))
            }

            Message::Flashcards(message) => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let Screen::Flashcards(flashcards) = screen else {
                    return Task::none();
                };

                match flashcards.update(message, database) {
                    flashcards::Action::None => Task::none(),
                    flashcards::Action::Run(task) => {
                        task.map(|msg| cosmic::action::app(Message::Flashcards(msg)))
                    }
                    flashcards::Action::OpenDeleteFlashcardDialog(flashcard) => {
                        self.update(Message::DialogAction(
                            dialogs::DialogAction::OpenDeleteFlashcardDialog(flashcard),
                        ))
                    }
                    flashcards::Action::OpenContextPage(context_page) => {
                        self.update(Message::ToggleContextPage(context_page))
                    }
                    flashcards::Action::StudyFolder(folder_id) => {
                        self.core.window.show_context = false;
                        self.update(Message::OpenStudy(folder_id))
                    }
                }
            }
            Message::OpenFlashcards(folder_id) => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let (flashcards, task) = screen::FlashcardsScreen::new(database, folder_id);
                *screen = Screen::Flashcards(flashcards);
                task.map(|msg| cosmic::action::app(Message::Flashcards(msg)))
            }

            Message::Study(message) => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let Screen::Study(study) = screen else {
                    return Task::none();
                };

                match study.update(message, database) {
                    study::Action::None => Task::none(),
                    study::Action::Back(folder_id) => {
                        self.update(Message::OpenFlashcards(folder_id))
                    }
                    study::Action::Run(task) => {
                        task.map(|msg| cosmic::action::app(Message::Study(msg)))
                    }
                }
            }
            Message::OpenStudy(folder_id) => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let (study, task) = screen::StudyScreen::new(database, folder_id);
                *screen = Screen::Study(study);
                task.map(|msg| cosmic::action::app(Message::Study(msg)))
            }
        }
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
            let message = Message::OpenFolders(*set_id);
            tasks.push(self.update(message));
            self.core.window.show_context = false;
        }

        tasks.push(self.update_title());
        Task::batch(tasks)
    }
}

impl AppModel {
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

    /// Settings context page
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
}
