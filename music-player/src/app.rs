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
    album_art_handle: Option<cosmic::iced::widget::image::Handle>,
    current_art_url: Option<String>,
}

impl Default for CosmicAppletMusic {
    fn default() -> Self {
        Self {
            core: Core::default(),
            popup: None,
            player_info: PlayerInfo::default(),
            music_controller: MusicController::new(),
            album_art_handle: None,
            current_art_url: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
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
        let app = CosmicAppletMusic {
            core,
            music_controller: MusicController::new(),
            ..Default::default()
        };
        (app, Task::done(cosmic::Action::App(Message::FindPlayer)))
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<Self::Message> {
        view::view(self)
    }

    fn view_window(&self, id: Id) -> Element<Self::Message> {
        view::view_window::view_window(self, id)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => self.handle_toggle_popup(),
            Message::PopupClosed(id) => self.handle_popup_closed(id),
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
        }
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
            Task::done(cosmic::Action::App(Message::FindPlayer))
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
            (None, Some(new_url)) => true,
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
        let _ = self.music_controller.find_active_player();
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
        Task::perform(async move {
            match reqwest::get(&url).await {
                Ok(response) => {
                    match response.bytes().await {
                        Ok(bytes) => {
                            let handle = cosmic::iced::widget::image::Handle::from_bytes(bytes);
                            Some(handle)
                        }
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }, |result| cosmic::Action::App(Message::AlbumArtLoaded(result)))
    }

    fn handle_album_art_loaded(&mut self, handle: Option<cosmic::iced::widget::image::Handle>) -> Task<Message> {
        self.album_art_handle = handle;
        Task::none()
    }
}
