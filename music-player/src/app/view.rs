use crate::app::{CosmicAppletMusic, Message};
use cosmic::widget::Id;
use cosmic::Element;
use mpris::PlaybackStatus;
use std::sync::LazyLock;

pub mod view_window;

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize-main"));

pub enum AppIcon {
    Playing,
    Paused,
}

impl AppIcon {
    fn to_str(&self) -> &'static str {
        match self {
            AppIcon::Playing => "media-playback-start-symbolic",
            AppIcon::Paused => "media-playback-pause-symbolic",
        }
    }
}

pub fn view(app: &CosmicAppletMusic) -> Element<'_, Message> {
    let icon = match app.player_info.status {
        PlaybackStatus::Playing => AppIcon::Paused, // Show pause when playing
        PlaybackStatus::Paused => AppIcon::Playing, // Show play when paused
        PlaybackStatus::Stopped => AppIcon::Playing, // Show play when stopped
    };

    use cosmic::iced::mouse;

    cosmic::widget::autosize::autosize(
        cosmic::widget::mouse_area(
            app.core
                .applet
                .icon_button(icon.to_str())
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
        .on_middle_press(Message::MiddleClick),
        AUTOSIZE_MAIN_ID.clone(),
    )
    .into()
}
