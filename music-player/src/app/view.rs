use crate::app::view::view_window::{get_album_cover, get_song_info};
use crate::app::{CosmicAppletMusic, Message};
use cosmic::iced::widget::{row,Space};
use cosmic::widget::{Id};
use cosmic::Element;
use mpris::PlaybackStatus;
use std::sync::LazyLock;

pub mod view_window;

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize-main"));

pub enum AppIcon {
    Playing,
    Paused,
    Next,
    Previous,
}

impl AppIcon {
    fn to_str(&self) -> &'static str {
        match self {
            AppIcon::Playing => "media-playback-start-symbolic",
            AppIcon::Paused => "media-playback-pause-symbolic",
            AppIcon::Next => "media-skip-forward-symbolic",
            AppIcon::Previous => "media-skip-backward-symbolic",
        }
    }
}

fn resolve_playback_icon(
    config_manager: Option<&crate::config::ConfigManager>,
    icon: &AppIcon,
) -> cosmic::widget::icon::Handle {
    let custom_path: Option<String> = config_manager.and_then(|config| match icon {
        AppIcon::Playing => config.get_custom_play_icon(),
        AppIcon::Paused => config.get_custom_pause_icon(),
        AppIcon::Next => config.get_custom_next_icon(),
        AppIcon::Previous => config.get_custom_previous_icon(),
    });

    match custom_path {
        Some(path) if !path.is_empty() => {
            cosmic::widget::icon::from_path(std::path::PathBuf::from(path))
        }
        _ => cosmic::widget::icon::from_name(icon.to_str())
            .symbolic(true)
            .into(),
    }
}

pub fn view(app: &CosmicAppletMusic) -> Element<'_, Message> {
    // Check if in multi-player mode
    let show_all_players = app
        .config_manager
        .as_ref()
        .map(|config| config.get_show_all_players())
        .unwrap_or(false);

    // Check if controls should be displayed
    let show_controls = app
        .config_manager
        .as_ref()
        .map(|config| config.get_show_controls())
        .unwrap_or(false);

    // Check if info pane should be displayed
    let show_info_pane = app
        .config_manager
        .as_ref()
        .map(|config| config.get_show_info_pane())
        .unwrap_or(false);

    // Gets info pane size
    let info_pane_width = app
        .config_manager
        .as_ref()
        .map(|config| config.get_info_pane_width())
        .unwrap_or(250);

    // Gets info pane position
    let info_pane_left = app
        .config_manager
        .as_ref()
        .map(|config| config.get_info_pane_left())
        .unwrap_or(true);

    // Gets ialbum miniature size
    let album_size = app
        .config_manager
        .as_ref()
        .map(|config| config.get_album_size())
        .unwrap_or(70);

    

    let icon = if show_all_players {
        // In multi-player mode, check if ANY player is playing
        let any_playing = app
            .all_players_info
            .iter()
            .any(|p| p.status == PlaybackStatus::Playing);

        if any_playing {
            AppIcon::Paused // Show pause when any player is playing
        } else {
            AppIcon::Playing // Show play when nothing is playing
        }
    } else {
        // Single-player mode
        match app.player_info.status {
            PlaybackStatus::Playing => AppIcon::Paused, // Show pause when playing
            PlaybackStatus::Paused => AppIcon::Playing, // Show play when paused
            PlaybackStatus::Stopped => AppIcon::Playing, // Show play when stopped
        }
    };

    use cosmic::iced::mouse;

    let icon_handle = resolve_playback_icon(app.config_manager.as_ref(), &icon);

    let back_button: Option<Element<'_, Message>> = if show_controls {
        let back_handle = resolve_playback_icon(app.config_manager.as_ref(), &AppIcon::Previous);
        Some(
            cosmic::widget::button::icon(back_handle)
                .icon_size(16)
                .padding(4)
                .on_press(Message::Previous)
                .into(),
        )
    }
    else {
        None
    };

    let skip_button: Option<Element<'_, Message>> = if show_controls {
        let skip_handle = resolve_playback_icon(app.config_manager.as_ref(), &AppIcon::Next);
        Some(
            cosmic::widget::button::icon(skip_handle)
                .icon_size(16)
                .padding(4)
                .on_press(Message::Next)
                .into(),
        )
    }
    else {
        None
    };

    let main_button = cosmic::widget::mouse_area(
                app.core
                    .applet
                    .icon_button_from_handle(icon_handle)
                    .on_press_down(Message::TogglePopup),
            )
            .on_scroll(|delta| match delta {
                mouse::ScrollDelta::Lines { y, .. } => {
                    if y > 0.0 {
                        Message::ScrollUp
                    } else {
                        Message::ScrollDown
                    }
                }
                mouse::ScrollDelta::Pixels { y, .. } => {
                    if y > 0.0 {
                        Message::ScrollUp
                    } else {
                        Message::ScrollDown
                    }
                }
            })
            .on_middle_press(Message::MiddleClick);

     let info_pane: Option<Element<'_, Message>> = if show_info_pane {
        let float_width = info_pane_width as f32;
        let float_size = album_size as f32;
        Some(row![
            get_song_info(app, 1.0)
            .padding(10)
            .height(float_size)
            .max_width(float_width - float_size),
            get_album_cover(app, float_size)
        ]
        .align_y(cosmic::iced::Alignment::Center)
        .width(float_width)
        .into()
        )
    } else {
        None
    };

    let (left_pane, right_pane) = if info_pane_left {
        (info_pane, None)
    } else {
        (None, info_pane)
    };
    
    // Automatic resize will not work in windowed mode,
    // but it works fine if the applet is in the dock/panel.
    // Windowed mode is not the goal of this app anyway.
    cosmic::widget::autosize::autosize(
        row![
            left_pane
            .unwrap_or_else(|| Space::new(0, 0).into()),
            back_button
            .unwrap_or_else(|| Space::new(0, 0).into()),
            main_button,
            skip_button
            .unwrap_or_else(|| Space::new(0, 0).into()),
            right_pane
            .unwrap_or_else(|| Space::new(0, 0).into())
        ]
        .align_y(cosmic::iced::Alignment::Center),
        AUTOSIZE_MAIN_ID.clone(),
    )
    .into()
}
