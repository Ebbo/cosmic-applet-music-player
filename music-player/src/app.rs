use crate::config::ConfigManager;
use crate::music::{MusicController, PlayerInfo};
use cosmic::app::{Core, Task};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::Limits;
use cosmic::{Application, Element};
use mpris::PlaybackStatus;

mod subscription;
mod view;

pub struct CosmicAppletMusic {
    core: Core,
    popup: Option<Id>,
    player_info: PlayerInfo,
    music_controller: MusicController,
    config_manager: Option<ConfigManager>,
    album_art_handle: Option<cosmic::iced::widget::image::Handle>,
    current_art_url: Option<String>,
    active_tab: PopupTab,
    all_players_info: Vec<PlayerInfo>,
    player_album_arts: std::collections::HashMap<String, cosmic::iced::widget::image::Handle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupTab {
    Controls,
    Settings,
}

impl Default for CosmicAppletMusic {
    fn default() -> Self {
        Self {
            core: Core::default(),
            popup: None,
            player_info: PlayerInfo::default(),
            music_controller: MusicController::new(),
            config_manager: None,
            album_art_handle: None,
            current_art_url: None,
            active_tab: PopupTab::Controls,
            all_players_info: Vec::new(),
            player_album_arts: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    SwitchTab(PopupTab),
    PlayPause,
    Next,
    Previous,
    UpdatePlayerInfo(PlayerInfo),
    FindPlayer,
    UpdateStatus(mpris::PlaybackStatus),
    VolumeChanged(f64),
    ScrollUp,
    ScrollDown,
    MiddleClick,
    LoadAlbumArt(String),
    AlbumArtLoaded(Option<cosmic::iced::widget::image::Handle>),
    DiscoverPlayers,
    ToggleAutoDetect(bool),
    SelectPlayer(Option<String>),
    UpdateAllPlayersInfo(Vec<PlayerInfo>),
    PlayPausePlayer(String),
    NextPlayer(String),
    PreviousPlayer(String),
    VolumeChangedPlayer(String, f64),
    LoadAlbumArtPlayer(String, String),
    AlbumArtLoadedPlayer(String, Option<cosmic::iced::widget::image::Handle>),
    ToggleShowAllPlayers(bool),
    ToggleHideInactive(bool),
}

impl Application for CosmicAppletMusic {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.cosmic.MusicPlayer";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let config_manager = ConfigManager::new().ok();
        let app = CosmicAppletMusic {
            core,
            music_controller: MusicController::new(),
            config_manager,
            active_tab: PopupTab::Controls,
            ..Default::default()
        };
        (
            app,
            Task::batch([
                Task::done(cosmic::Action::App(Message::DiscoverPlayers)),
                Task::done(cosmic::Action::App(Message::FindPlayer)),
            ]),
        )
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        view::view(self)
    }

    fn view_window(&self, id: Id) -> Element<'_, Self::Message> {
        view::view_window::view_window(self, id)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => self.handle_toggle_popup(),
            Message::PopupClosed(id) => self.handle_popup_closed(id),
            Message::SwitchTab(tab) => self.handle_switch_tab(tab),
            Message::PlayPause => self.handle_play_pause(),
            Message::Next => self.handle_next(),
            Message::Previous => self.handle_previous(),
            Message::UpdatePlayerInfo(info) => self.handle_update_player_info(info),
            Message::FindPlayer => self.handle_find_player(),
            Message::UpdateStatus(status) => self.handle_update_status(status),
            Message::VolumeChanged(volume) => self.handle_volume_changed(volume),
            Message::ScrollUp => self.handle_next(),
            Message::ScrollDown => self.handle_previous(),
            Message::MiddleClick => self.handle_play_pause(),
            Message::LoadAlbumArt(url) => self.handle_load_album_art(url),
            Message::AlbumArtLoaded(handle) => self.handle_album_art_loaded(handle),
            Message::DiscoverPlayers => self.handle_discover_players(),
            Message::ToggleAutoDetect(enabled) => self.handle_toggle_auto_detect(enabled),
            Message::SelectPlayer(player) => self.handle_select_player(player),
            Message::UpdateAllPlayersInfo(info) => self.handle_update_all_players_info(info),
            Message::PlayPausePlayer(bus_name) => self.handle_play_pause_player(bus_name),
            Message::NextPlayer(bus_name) => self.handle_next_player(bus_name),
            Message::PreviousPlayer(bus_name) => self.handle_previous_player(bus_name),
            Message::VolumeChangedPlayer(bus_name, volume) => {
                self.handle_volume_changed_player(bus_name, volume)
            }
            Message::LoadAlbumArtPlayer(bus_name, url) => {
                self.handle_load_album_art_player(bus_name, url)
            }
            Message::AlbumArtLoadedPlayer(bus_name, handle) => {
                self.handle_album_art_loaded_player(bus_name, handle)
            }
            Message::ToggleShowAllPlayers(enabled) => self.handle_toggle_show_all_players(enabled),
            Message::ToggleHideInactive(enabled) => self.handle_toggle_hide_inactive(enabled),
        }
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        subscription::subscription()
    }
}

impl CosmicAppletMusic {
    fn handle_toggle_popup(&mut self) -> Task<Message> {
        if let Some(p) = self.popup.take() {
            destroy_popup(p)
        } else {
            let new_id = Id::unique();
            self.popup.replace(new_id);
            let mut popup_settings = self.core.applet.get_popup_settings(
                self.core.main_window_id().unwrap(),
                new_id,
                None,
                None,
                None,
            );
            popup_settings.positioner.size_limits = Limits::NONE
                .max_width(400.0)
                .min_width(300.0)
                .min_height(150.0)
                .max_height(300.0);
            get_popup(popup_settings)
        }
    }

    fn handle_popup_closed(&mut self, id: Id) -> Task<Message> {
        if self.popup.as_ref() == Some(&id) {
            self.popup = None;
            // Reset to controls tab when popup closes
            self.active_tab = PopupTab::Controls;
        }
        Task::none()
    }

    fn handle_switch_tab(&mut self, tab: PopupTab) -> Task<Message> {
        self.active_tab = tab;
        Task::none()
    }

    fn handle_play_pause(&self) -> Task<Message> {
        let _ = self.music_controller.play_pause();

        // Immediately toggle the UI status for responsive feedback
        let new_status = match self.player_info.status {
            PlaybackStatus::Playing => PlaybackStatus::Paused,
            PlaybackStatus::Paused | PlaybackStatus::Stopped => PlaybackStatus::Playing,
        };

        Task::batch([
            Task::done(cosmic::Action::App(Message::UpdateStatus(new_status))),
            Task::done(cosmic::Action::App(Message::FindPlayer)),
        ])
    }

    fn handle_next(&self) -> Task<Message> {
        let _ = self.music_controller.next();
        Task::done(cosmic::Action::App(Message::FindPlayer))
    }

    fn handle_previous(&self) -> Task<Message> {
        let _ = self.music_controller.previous();
        Task::done(cosmic::Action::App(Message::FindPlayer))
    }

    fn handle_update_player_info(&mut self, info: PlayerInfo) -> Task<Message> {
        // Check if album art URL changed
        let should_load_art = match (&self.current_art_url, &info.art_url) {
            (None, Some(_new_url)) => true,
            (Some(old_url), Some(new_url)) => old_url != new_url,
            (Some(_), None) => {
                self.album_art_handle = None;
                self.current_art_url = None;
                false
            }
            (None, None) => false,
        };

        self.player_info = info.clone();

        if should_load_art {
            if let Some(url) = info.art_url {
                self.current_art_url = Some(url.clone());
                return Task::done(cosmic::Action::App(Message::LoadAlbumArt(url)));
            }
        }

        Task::none()
    }

    fn handle_find_player(&mut self) -> Task<Message> {
        // Check if in multi-player mode
        let show_all_players = self
            .config_manager
            .as_ref()
            .map(|config| config.get_show_all_players())
            .unwrap_or(false);

        if show_all_players {
            // In multi-player mode, update all players
            let _ = self.music_controller.discover_all_players();
            let all_players = self.music_controller.get_all_players_info();
            return Task::done(cosmic::Action::App(Message::UpdateAllPlayersInfo(
                all_players,
            )));
        }

        // Single-player mode
        if let Some(ref config) = self.config_manager {
            // Use new selected player approach
            if let Some(selected_player) = config.get_selected_player() {
                let _ = self.music_controller.find_specific_player(&selected_player);
            } else {
                // No player selected - try to find any active player for backward compatibility
                let _ = self.music_controller.find_active_player();
            }
        } else {
            let _ = self.music_controller.find_active_player();
        }
        let info = self.music_controller.get_player_info();
        Task::done(cosmic::Action::App(Message::UpdatePlayerInfo(info)))
    }

    fn handle_update_status(&mut self, status: PlaybackStatus) -> Task<Message> {
        self.player_info.status = status;
        Task::none()
    }

    fn handle_volume_changed(&mut self, volume: f64) -> Task<Message> {
        let _ = self.music_controller.set_volume(volume);
        self.player_info.volume = volume;
        Task::none()
    }

    fn handle_load_album_art(&mut self, url: String) -> Task<Message> {
        Task::perform(
            async move {
                match reqwest::get(&url).await {
                    Ok(response) => match response.bytes().await {
                        Ok(bytes) => {
                            let handle = cosmic::iced::widget::image::Handle::from_bytes(bytes);
                            Some(handle)
                        }
                        Err(_) => None,
                    },
                    Err(_) => None,
                }
            },
            |result| cosmic::Action::App(Message::AlbumArtLoaded(result)),
        )
    }

    fn handle_album_art_loaded(
        &mut self,
        handle: Option<cosmic::iced::widget::image::Handle>,
    ) -> Task<Message> {
        self.album_art_handle = handle;
        Task::none()
    }

    fn handle_discover_players(&mut self) -> Task<Message> {
        let _ = self.music_controller.discover_all_players();

        // Auto-add discovered players to config if auto-detect is enabled
        if let Some(ref mut config) = self.config_manager {
            let discovered = self.music_controller.get_discovered_players();
            for player in discovered {
                let _ = config.add_discovered_player(player.identity);
            }
        }

        Task::none()
    }

    fn handle_toggle_auto_detect(&mut self, enabled: bool) -> Task<Message> {
        if let Some(ref mut config) = self.config_manager {
            let _ = config.set_auto_detect_new_players(enabled);
        }
        Task::none()
    }

    fn handle_select_player(&mut self, player: Option<String>) -> Task<Message> {
        if let Some(ref mut config) = self.config_manager {
            let _ = config.set_selected_player(player);
        }
        Task::done(cosmic::Action::App(Message::FindPlayer))
    }

    fn handle_update_all_players_info(&mut self, players_info: Vec<PlayerInfo>) -> Task<Message> {
        // Update the list of all players
        self.all_players_info = players_info.clone();

        // Load album arts for new players
        let mut tasks = Vec::new();
        for player in players_info {
            if let Some(ref art_url) = player.art_url {
                if !self.player_album_arts.contains_key(&player.bus_name) {
                    let bus_name = player.bus_name.clone();
                    let url = art_url.clone();
                    tasks.push(Task::done(cosmic::Action::App(
                        Message::LoadAlbumArtPlayer(bus_name, url),
                    )));
                }
            }
        }

        Task::batch(tasks)
    }

    fn handle_play_pause_player(&mut self, bus_name: String) -> Task<Message> {
        let _ = self.music_controller.play_pause_player(&bus_name);

        // Update the player info
        Task::batch([
            Task::done(cosmic::Action::App(Message::DiscoverPlayers)),
            Task::done(cosmic::Action::App(Message::UpdateAllPlayersInfo(
                self.music_controller.get_all_players_info(),
            ))),
        ])
    }

    fn handle_next_player(&mut self, bus_name: String) -> Task<Message> {
        let _ = self.music_controller.next_player(&bus_name);
        Task::batch([
            Task::done(cosmic::Action::App(Message::DiscoverPlayers)),
            Task::done(cosmic::Action::App(Message::UpdateAllPlayersInfo(
                self.music_controller.get_all_players_info(),
            ))),
        ])
    }

    fn handle_previous_player(&mut self, bus_name: String) -> Task<Message> {
        let _ = self.music_controller.previous_player(&bus_name);
        Task::batch([
            Task::done(cosmic::Action::App(Message::DiscoverPlayers)),
            Task::done(cosmic::Action::App(Message::UpdateAllPlayersInfo(
                self.music_controller.get_all_players_info(),
            ))),
        ])
    }

    fn handle_volume_changed_player(&mut self, bus_name: String, volume: f64) -> Task<Message> {
        let _ = self.music_controller.set_volume_player(&bus_name, volume);

        // Update the player info in the list
        if let Some(player) = self
            .all_players_info
            .iter_mut()
            .find(|p| p.bus_name == bus_name)
        {
            player.volume = volume;
        }

        Task::none()
    }

    fn handle_load_album_art_player(&mut self, bus_name: String, url: String) -> Task<Message> {
        Task::perform(
            async move {
                match reqwest::get(&url).await {
                    Ok(response) => match response.bytes().await {
                        Ok(bytes) => {
                            let handle = cosmic::iced::widget::image::Handle::from_bytes(bytes);
                            (bus_name, Some(handle))
                        }
                        Err(_) => (bus_name, None),
                    },
                    Err(_) => (bus_name, None),
                }
            },
            |(bus_name, handle)| {
                cosmic::Action::App(Message::AlbumArtLoadedPlayer(bus_name, handle))
            },
        )
    }

    fn handle_album_art_loaded_player(
        &mut self,
        bus_name: String,
        handle: Option<cosmic::iced::widget::image::Handle>,
    ) -> Task<Message> {
        if let Some(handle) = handle {
            self.player_album_arts.insert(bus_name, handle);
        }
        Task::none()
    }

    fn handle_toggle_show_all_players(&mut self, enabled: bool) -> Task<Message> {
        if let Some(ref mut config) = self.config_manager {
            let _ = config.set_show_all_players(enabled);
        }

        // If enabling, discover and update all players
        if enabled {
            Task::batch([
                Task::done(cosmic::Action::App(Message::DiscoverPlayers)),
                Task::done(cosmic::Action::App(Message::UpdateAllPlayersInfo(
                    self.music_controller.get_all_players_info(),
                ))),
            ])
        } else {
            Task::none()
        }
    }

    fn handle_toggle_hide_inactive(&mut self, enabled: bool) -> Task<Message> {
        if let Some(ref mut config) = self.config_manager {
            let _ = config.set_hide_inactive_players(enabled);
        }
        Task::none()
    }
}
